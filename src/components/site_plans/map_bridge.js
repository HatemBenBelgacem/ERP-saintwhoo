// Kartenbrücke des Geländeplans (ANF-027): MapLibre GL JS zeichnet Satellitenbild und Karte,
// Three.js zeichnet im selben <canvas> die 3D-Elemente darüber (eigene Ebene "custom" von MapLibre,
// die die Kameramatrix der Karte an Three.js weitergibt).
//
// Die Bibliotheken werden erst beim Öffnen eines Geländeplans als ES-Module geladen (kein npm,
// kein Bündeln) – alle anderen Seiten des ERP laden sie nie. Gesteuert wird alles aus Rust
// (src/components/site_plans/bridge.rs).
//
// Koordinaten der Szene: Meter relativ zum Ursprung, x nach Osten, z nach Süden, y nach oben.

const MAPLIBRE_URL = 'https://esm.sh/maplibre-gl@4.7.1';
const THREE_URL = 'https://esm.sh/three@0.170.0';
const MAPLIBRE_CSS = 'https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css';

const EARTH_RADIUS_M = 6378137.0;

// Ausdehnung von SWISSIMAGE (Schweiz und Liechtenstein) als [West, Süd, Ost, Nord]; gleiche Werte wie
// SWISS_BOUNDS in src/backend/site_plans/model.rs
const SWISS_BOUNDS = [5.9, 45.8, 10.5, 47.9];

let libraries = null;

// Lädt MapLibre (mit Stilvorlage) und Three.js einmal je Seitenaufruf.
function loadLibraries() {
    if (!libraries) {
        if (!document.querySelector(`link[href="${MAPLIBRE_CSS}"]`)) {
            const link = document.createElement('link');
            link.rel = 'stylesheet';
            link.href = MAPLIBRE_CSS;
            document.head.appendChild(link);
        }
        libraries = Promise.all([import(MAPLIBRE_URL), import(THREE_URL)]).then(
            ([maplibre, three]) => ({ maplibregl: maplibre.default ?? maplibre, THREE: three })
        );
        // Bei einem Fehler beim nächsten Öffnen erneut versuchen
        libraries.catch(() => { libraries = null; });
    }
    return libraries;
}

// Farbe aus dem Farbschema (styles/app.css), z. B. --gelaende-buehne
function themeColor(THREE, name, fallback) {
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    const color = new THREE.Color(fallback);
    if (value) {
        try { color.setStyle(value); } catch (_) { /* Vorgabe behalten */ }
    }
    return color;
}

export class SitePlanMap {
    // container: HTML-Element der Karte; onEvent(ereignis) meldet Bedienungen an Rust (Meter relativ
    // zum Ursprung):
    //   { type: 'click', x, z }                    Klick auf eine freie Stelle
    //   { type: 'pick', index }                     Element angeklickt bzw. Ziehen begonnen
    //   { type: 'drag', index, x, z }               Element wird gezogen (neuer Mittelpunkt)
    //   { type: 'drop', index, start_x, start_z }   Ziehen beendet (mit Ausgangslage)
    //   { type: 'rotate', index, deg }              Element wird am Drehgriff gedreht (neuer Winkel)
    //   { type: 'rotated', index, start_deg }       Drehen beendet (mit Ausgangswinkel)
    //   { type: 'corner', index, x, z }             Eckpunkt der Rasenfläche wird gezogen
    // onError(text) meldet, wenn die Karte nicht geladen werden konnte.
    constructor(container, originLng, originLat, onEvent, onError) {
        this.origin = [originLng, originLat];
        this.onEvent = onEvent;
        this.drag = null; // laufendes Ziehen eines Elements
        this.suppressClick = false; // Klick nach dem Anfassen eines Elements nicht als Platzieren werten
        this.elementModels = [];
        this.scene = null; // zuletzt gemeldeter Inhalt aus Rust
        this.destroyed = false;
        this.map = null;

        loadLibraries()
            .then((libs) => {
                if (!this.destroyed) this.#setup(container, libs);
            })
            .catch((e) => {
                if (!this.destroyed) onError(`Die Karte konnte nicht geladen werden (${e?.message ?? e}).`);
            });
    }

    #setup(container, { maplibregl, THREE }) {
        this.maplibregl = maplibregl;
        this.THREE = THREE;
        this.#computeTransform();

        const map = new maplibregl.Map({
            container,
            center: this.origin,
            zoom: 17,
            pitch: 60, // Kamera geneigt: 3D-Ansicht
            bearing: -20,
            maxPitch: 80,
            antialias: true,
            attributionControl: { compact: true },
            style: {
                version: 8,
                sources: {
                    // Amtliches Luftbild der Schweiz: swisstopo SWISSIMAGE (rund 10 cm pro Bildpunkt),
                    // frei nutzbar mit Quellenangabe. Deckt nur die Schweiz und Liechtenstein ab (bounds);
                    // ausserhalb bleibt der Hintergrund sichtbar. Über Zoomstufe 20 wird vergrössert.
                    luftbild: {
                        type: 'raster',
                        tiles: [
                            'https://wmts.geo.admin.ch/1.0.0/ch.swisstopo.swissimage/default/current/3857/{z}/{x}/{y}.jpeg'
                        ],
                        tileSize: 256,
                        maxzoom: 20,
                        bounds: SWISS_BOUNDS,
                        attribution: '<a href="https://www.swisstopo.admin.ch/" target="_blank" rel="noopener">Luftbild © swisstopo</a>'
                    }
                },
                layers: [
                    { id: 'hintergrund', type: 'background', paint: { 'background-color': cssColor('--color-base-300', '#e2e8f0') } },
                    { id: 'luftbild', type: 'raster', source: 'luftbild' }
                ]
            }
        });
        this.map = map;
        map.getCanvas().style.cursor = 'crosshair';
        map.addControl(new maplibregl.NavigationControl({ visualizePitch: true }), 'top-right');
        map.addControl(new maplibregl.ScaleControl({ unit: 'metric' }), 'bottom-left');

        const self = this;
        const layer = {
            id: 'gelaendeplan-3d',
            type: 'custom',
            renderingMode: '3d',

            onAdd(mapInstance, gl) {
                self.camera = new THREE.Camera();
                self.world = new THREE.Scene();
                self.group = new THREE.Group();
                self.world.add(self.group);

                // Licht: Grundhelligkeit und Sonne für Tiefe
                self.world.add(new THREE.AmbientLight(0xffffff, 0.7));
                const sun = new THREE.DirectionalLight(0xffffff, 0.8);
                sun.position.set(50, 100, 50);
                self.world.add(sun);

                // Three.js nutzt den WebGL-Kontext von MapLibre
                self.renderer = new THREE.WebGLRenderer({
                    canvas: mapInstance.getCanvas(),
                    context: gl,
                    antialias: true
                });
                self.renderer.autoClear = false; // Karte nicht übermalen
                self.#rebuild();
            },

            render(gl, args) {
                // MapLibre 4 übergibt die Matrix direkt, neuere Fassungen in einem Objekt
                const matrix = args?.defaultProjectionData?.mainMatrix ?? args;
                const t = self.transform;
                const rotationX = new THREE.Matrix4().makeRotationAxis(new THREE.Vector3(1, 0, 0), Math.PI / 2);
                const m = new THREE.Matrix4().fromArray(matrix);
                const l = new THREE.Matrix4()
                    .makeTranslation(t.x, t.y, t.z)
                    .scale(new THREE.Vector3(t.scale, -t.scale, t.scale))
                    .multiply(rotationX);
                self.camera.projectionMatrix = m.multiply(l);
                self.renderer.resetState();
                self.renderer.render(self.world, self.camera);
            }
        };

        // Reihenfolge der Ebenen: Rasenfläche (unter den Modellen), 3D-Modelle, Eckpunkte, Messung, Drehgriff
        map.on('style.load', () => {
            this.#addAreaLayers();
            this.#addFlowLayers();
            map.addLayer(layer);
            this.#addCornerLayer();
            this.#addMeasureLayers();
            this.#addHandleLayers();
            this.#updateOverlays();
        });

        // Klick auf eine freie Stelle in Meter relativ zum Ursprung umrechnen und an Rust melden
        map.on('click', (e) => {
            if (this.suppressClick) {
                this.suppressClick = false;
                return;
            }
            const { x, z } = this.#toLocal(e.lngLat.lng, e.lngLat.lat);
            this.onEvent({ type: 'click', x, z });
        });

        // Ziehen mit Maus oder einem Finger – die Karte bewegt sich dabei nicht:
        //   Drehgriff des ausgewählten Elements → Element um die eigene Achse drehen
        //   Eckpunkt der Rasenfläche            → Eckpunkt verschieben
        //   Element                             → Element verschieben (folgt dem Bodenpunkt unter dem Zeiger)
        const start = (e) => {
            this.suppressClick = false;
            if (!this.scene) return;
            if (e.type === 'touchstart' && e.points.length !== 1) return;
            const target = this.#target(e.point);
            if (!target) return;
            e.preventDefault(); // kein Verschieben oder Drehen der Karte
            this.suppressClick = true;
            const p = this.#toLocal(e.lngLat.lng, e.lngLat.lat);
            if (target.kind === 'element') {
                const el = this.scene.elements[target.index];
                this.drag = { ...target, startX: el.x, startZ: el.z, grabX: p.x - el.x, grabZ: p.z - el.z, moved: false };
                this.onEvent({ type: 'pick', index: target.index });
            } else if (target.kind === 'rotate') {
                this.drag = { ...target, startDeg: this.scene.elements[target.index].rotation_deg, moved: false };
            } else {
                const [cx, cz] = this.scene.corners[target.index];
                this.drag = { ...target, grabX: p.x - cx, grabZ: p.z - cz, moved: false };
            }
        };
        const move = (e) => {
            const d = this.drag;
            if (!d) {
                // Zeiger zeigt, was sich unter ihm anfassen lässt
                if (e.type === 'mousemove') {
                    const t = this.#target(e.point);
                    map.getCanvas().style.cursor = !t ? 'crosshair' : t.kind === 'rotate' ? 'grab' : 'move';
                }
                return;
            }
            const p = this.#toLocal(e.lngLat.lng, e.lngLat.lat);
            d.moved = true;
            if (d.kind === 'element') {
                this.onEvent({ type: 'drag', index: d.index, x: p.x - d.grabX, z: p.z - d.grabZ });
            } else if (d.kind === 'rotate') {
                // Winkel vom Mittelpunkt zum Zeiger; ganze Grad, mit Umschalt in 15°-Schritten
                const el = this.scene.elements[d.index];
                let deg = Math.atan2(p.x - el.x, p.z - el.z) * 180 / Math.PI;
                deg = e.originalEvent?.shiftKey ? Math.round(deg / 15) * 15 : Math.round(deg);
                deg = ((deg % 360) + 360) % 360;
                this.onEvent({ type: 'rotate', index: d.index, deg });
            } else {
                this.onEvent({ type: 'corner', index: d.index, x: p.x - d.grabX, z: p.z - d.grabZ });
            }
        };
        this.endDrag = () => {
            const d = this.drag;
            if (!d) return;
            this.drag = null;
            if (!d.moved) return;
            if (d.kind === 'element') {
                this.onEvent({ type: 'drop', index: d.index, start_x: d.startX, start_z: d.startZ });
            } else if (d.kind === 'rotate') {
                this.onEvent({ type: 'rotated', index: d.index, start_deg: d.startDeg });
            }
        };
        map.on('mousedown', start);
        map.on('touchstart', start);
        map.on('mousemove', move);
        map.on('touchmove', move);
        map.on('mouseup', this.endDrag);
        map.on('touchend', this.endDrag);
        map.on('touchcancel', this.endDrag);
        // Loslassen ausserhalb der Karte beendet das Ziehen ebenfalls
        window.addEventListener('mouseup', this.endDrag);
    }

    // Was liegt unter dem Bildschirmpunkt? Drehgriff und Eckpunkte werden in Bildschirmpixeln
    // gesucht (gut zu treffen in jeder Zoomstufe), Elemente mit einem Strahl durch die 3D-Szene.
    #target(point) {
        const s = this.scene;
        if (!s || !this.map) return null;
        const near = (x, z, radius) => {
            const q = this.map.project(this.#toLngLat(x, z));
            return Math.hypot(q.x - point.x, q.y - point.y) <= radius;
        };
        if (s.drag_elements) {
            const index = s.elements.findIndex((el) => el.selected);
            if (index >= 0) {
                const [hx, hz] = handlePosition(s.elements[index]);
                if (near(hx, hz, 14)) return { kind: 'rotate', index };
            }
        }
        if (s.drag_corners) {
            const index = s.corners.findIndex(([x, z]) => near(x, z, 12));
            if (index >= 0) return { kind: 'corner', index };
        }
        if (s.drag_elements) {
            const index = this.#pick(point);
            if (index >= 0) return { kind: 'element', index };
        }
        return null;
    }

    // Element unter einem Bildschirmpunkt (Index in der Liste aus Rust) oder -1. Der Strahl durch
    // den Punkt wird mit der umgekehrten Kameramatrix in Meter-Koordinaten der Szene umgerechnet.
    #pick(point) {
        if (!this.camera || this.elementModels.length === 0) return -1;
        const THREE = this.THREE;
        const canvas = this.map.getCanvas();
        const nx = (point.x / canvas.clientWidth) * 2 - 1;
        const ny = 1 - (point.y / canvas.clientHeight) * 2;
        const inverse = this.camera.projectionMatrix.clone().invert();
        const near = new THREE.Vector3(nx, ny, -1).applyMatrix4(inverse);
        const far = new THREE.Vector3(nx, ny, 1).applyMatrix4(inverse);
        this.raycaster ??= new THREE.Raycaster();
        this.raycaster.set(near, far.sub(near).normalize());
        this.group.updateMatrixWorld(true);
        for (const hit of this.raycaster.intersectObjects(this.elementModels, true)) {
            let o = hit.object;
            while (o && o.userData.index === undefined) o = o.parent;
            if (o) return o.userData.index;
        }
        return -1;
    }

    // Umrechnung vom Ursprung in Mercator-Koordinaten von MapLibre
    #computeTransform() {
        const mc = this.maplibregl.MercatorCoordinate.fromLngLat(this.origin, 0);
        this.transform = { x: mc.x, y: mc.y, z: mc.z, scale: mc.meterInMercatorCoordinateUnits() };
    }

    // GPS-Koordinaten → Meter relativ zum Ursprung (Näherung für ein Gelände von einigen Kilometern)
    #toLocal(lng, lat) {
        const [oLng, oLat] = this.origin;
        const dLat = (lat - oLat) * Math.PI / 180;
        const dLng = (lng - oLng) * Math.PI / 180;
        return {
            x: dLng * EARTH_RADIUS_M * Math.cos(oLat * Math.PI / 180),
            z: -dLat * EARTH_RADIUS_M
        };
    }

    // Meter relativ zum Ursprung → [Längengrad, Breitengrad] (Umkehrung von #toLocal)
    #toLngLat(x, z) {
        const [oLng, oLat] = this.origin;
        const lng = oLng + (x / (EARTH_RADIUS_M * Math.cos(oLat * Math.PI / 180))) * 180 / Math.PI;
        const lat = oLat - (z / EARTH_RADIUS_M) * 180 / Math.PI;
        return [lng, lat];
    }

    // Rasenfläche als Ebenen von MapLibre: Füllung und Umriss liegen unter den 3D-Modellen.
    #addAreaLayers() {
        const color = cssColor('--gelaende-flaeche', '#a855f7');
        this.map.addSource('flaeche', { type: 'geojson', data: emptyCollection() });
        this.map.addLayer({
            id: 'flaeche-fuellung', type: 'fill', source: 'flaeche',
            filter: ['==', '$type', 'Polygon'],
            paint: { 'fill-color': color, 'fill-opacity': 0.35 }
        });
        this.map.addLayer({
            id: 'flaeche-linie', type: 'line', source: 'flaeche',
            filter: ['==', '$type', 'LineString'],
            paint: { 'line-color': color, 'line-width': 2.5 }
        });
    }

    // Eckpunkte über den Modellen, damit man sie immer anfassen kann
    #addCornerLayer() {
        this.map.addLayer({
            id: 'flaeche-ecke', type: 'circle', source: 'flaeche',
            filter: ['==', '$type', 'Point'],
            paint: {
                'circle-color': cssColor('--gelaende-flaeche', '#a855f7'),
                'circle-radius': 6,
                'circle-stroke-color': cssColor('--gelaende-auswahl', '#ffffff'),
                'circle-stroke-width': 2
            }
        });
    }

    // Drehgriff des ausgewählten Elements: Linie vom Mittelpunkt zum Griff und der Griff selbst
    #addHandleLayers() {
        const color = cssColor('--gelaende-auswahl', '#ffffff');
        this.map.addSource('drehgriff', { type: 'geojson', data: emptyCollection() });
        this.map.addLayer({
            id: 'drehgriff-linie', type: 'line', source: 'drehgriff',
            filter: ['==', '$type', 'LineString'],
            paint: { 'line-color': color, 'line-width': 2 }
        });
        this.map.addLayer({
            id: 'drehgriff-punkt', type: 'circle', source: 'drehgriff',
            filter: ['==', '$type', 'Point'],
            paint: { 'circle-color': color, 'circle-radius': 8, 'circle-stroke-color': '#000', 'circle-stroke-width': 1.5 }
        });
    }

    // Rasenfläche, Drehgriff und Messung aus dem zuletzt gemeldeten Inhalt zeichnen.
    #updateOverlays() {
        const area = this.map?.getSource('flaeche');
        const handle = this.map?.getSource('drehgriff');
        if (!area || !handle || !this.scene) return;

        const coords = this.scene.corners.map(([x, z]) => this.#toLngLat(x, z));
        const features = coords.map((c, index) => point(c, { index }));
        if (coords.length >= 2) {
            features.push(lineString(coords.length >= 3 ? [...coords, coords[0]] : coords));
        }
        if (coords.length >= 3) {
            features.push({ type: 'Feature', geometry: { type: 'Polygon', coordinates: [[...coords, coords[0]]] }, properties: {} });
        }
        area.setData({ type: 'FeatureCollection', features });

        const selected = this.scene.drag_elements ? this.scene.elements.find((el) => el.selected) : null;
        if (selected) {
            const center = this.#toLngLat(selected.x, selected.z);
            const grip = this.#toLngLat(...handlePosition(selected));
            handle.setData({ type: 'FeatureCollection', features: [lineString([center, grip]), point(grip, {})] });
        } else {
            handle.setData(emptyCollection());
        }

        this.#updateMeasure();
        this.#updateFlows();
    }

    // Besucherströme (ANF-028) als Ebenen von MapLibre, unter den 3D-Modellen:
    // Zonen gefüllt in der Farbe ihrer Ampel, Wege umso breiter, je mehr Personen pro Stunde,
    // bei Einbahn mit weissen Pfeilen in Wegrichtung.
    #addFlowLayers() {
        const zoneColor = ['match', ['get', 'status'],
            'ok', cssColor('--color-success', '#15803d'),
            'knapp', cssColor('--color-warning', '#f59e0b'),
            'voll', cssColor('--color-error', '#dc2626'),
            cssColor('--color-base-content', '#1e293b')];
        this.map.addSource('zonen', { type: 'geojson', data: emptyCollection() });
        this.map.addLayer({
            id: 'zonen-fuellung', type: 'fill', source: 'zonen',
            filter: ['==', '$type', 'Polygon'],
            paint: { 'fill-color': zoneColor, 'fill-opacity': 0.3 }
        });
        this.map.addLayer({
            id: 'zonen-linie', type: 'line', source: 'zonen',
            filter: ['==', '$type', 'LineString'],
            paint: { 'line-color': zoneColor, 'line-width': ['case', ['get', 'active'], 3, 2], 'line-dasharray': [3, 2] }
        });
        this.map.addLayer({
            id: 'zonen-punkt', type: 'circle', source: 'zonen',
            filter: ['all', ['==', '$type', 'Point'], ['==', 'active', true]],
            paint: { 'circle-color': zoneColor, 'circle-radius': 4, 'circle-stroke-color': '#fff', 'circle-stroke-width': 1.5 }
        });

        const routeColor = cssColor('--gelaende-besucherweg', '#0891b2');
        this.map.addSource('besucherwege', { type: 'geojson', data: emptyCollection() });
        this.map.addLayer({
            id: 'besucherwege-linie', type: 'line', source: 'besucherwege',
            filter: ['==', '$type', 'LineString'],
            layout: { 'line-cap': 'round', 'line-join': 'round' },
            paint: {
                'line-color': routeColor,
                'line-opacity': 0.85,
                // Breite in Bildschirmpixeln je nach Personen pro Stunde
                'line-width': ['interpolate', ['linear'], ['get', 'flow'], 0, 4, 500, 7, 2000, 12, 10000, 20]
            }
        });
        this.map.addImage('pfeil', arrowImage(), { pixelRatio: 2 });
        this.map.addLayer({
            id: 'besucherwege-pfeil', type: 'symbol', source: 'besucherwege',
            filter: ['all', ['==', '$type', 'LineString'], ['==', 'one_way', true]],
            layout: {
                'symbol-placement': 'line',
                'symbol-spacing': 70,
                'icon-image': 'pfeil',
                'icon-allow-overlap': true,
                'icon-ignore-placement': true,
                'icon-rotation-alignment': 'map'
            }
        });
        this.map.addLayer({
            id: 'besucherwege-punkt', type: 'circle', source: 'besucherwege',
            filter: ['all', ['==', '$type', 'Point'], ['==', 'active', true]],
            paint: { 'circle-color': routeColor, 'circle-radius': 4, 'circle-stroke-color': '#fff', 'circle-stroke-width': 1.5 }
        });
    }

    // Wege, Zonen und Zugänge aus dem zuletzt gemeldeten Inhalt zeichnen und beschriften.
    #updateFlows() {
        for (const m of this.flowMarkers ?? []) m.remove();
        this.flowMarkers = [];
        const zones = this.map?.getSource('zonen');
        const routes = this.map?.getSource('besucherwege');
        if (!zones || !routes || !this.scene) return;
        const label = (text, [x, z], className) => {
            const el = document.createElement('div');
            el.className = className;
            el.textContent = text;
            this.flowMarkers.push(new this.maplibregl.Marker({ element: el })
                .setLngLat(this.#toLngLat(x, z)).addTo(this.map));
        };

        const zoneFeatures = [];
        for (const zone of this.scene.zones ?? []) {
            const props = { status: zone.status, active: zone.active };
            const coords = zone.points.map(([x, z]) => this.#toLngLat(x, z));
            if (zone.active) for (const c of coords) zoneFeatures.push(point(c, props));
            if (coords.length >= 2) {
                const line = lineString(coords.length >= 3 ? [...coords, coords[0]] : coords);
                line.properties = props;
                zoneFeatures.push(line);
            }
            if (coords.length >= 3) {
                zoneFeatures.push({ type: 'Feature', geometry: { type: 'Polygon', coordinates: [[...coords, coords[0]]] }, properties: props });
            }
            label(zone.label, zone.label_at, 'badge badge-sm pointer-events-none border-base-300 bg-base-100/90 font-semibold text-base-content shadow-sm');
        }
        zones.setData({ type: 'FeatureCollection', features: zoneFeatures });

        const routeFeatures = [];
        for (const route of this.scene.routes ?? []) {
            const props = { flow: route.flow, one_way: route.one_way, active: route.active };
            const coords = route.points.map(([x, z]) => this.#toLngLat(x, z));
            if (route.active) for (const c of coords) routeFeatures.push(point(c, props));
            if (coords.length >= 2) {
                const line = lineString(coords);
                line.properties = props;
                routeFeatures.push(line);
            }
            label(route.label, route.label_at, 'badge badge-sm pointer-events-none border-0 bg-site-route text-white shadow-sm');
        }
        routes.setData({ type: 'FeatureCollection', features: routeFeatures });

        // Zugänge: Kurzzeichen im farbigen Kreis und Bezeichnung
        const accessClass = {
            eingang: 'bg-site-entrance',
            ausgang: 'bg-site-exit',
            notausgang: 'bg-site-emergency-exit'
        };
        for (const access of this.scene.accesses ?? []) {
            const el = document.createElement('div');
            el.className = 'pointer-events-none flex items-center gap-1';
            const sign = document.createElement('span');
            sign.className = `flex size-6 items-center justify-center rounded-full border-2 border-white text-xs font-bold text-white shadow ${accessClass[access.kind] ?? 'bg-neutral'}`;
            sign.textContent = access.short;
            const name = document.createElement('span');
            name.className = 'badge badge-sm border-base-300 bg-base-100/90 text-base-content shadow-sm';
            name.textContent = access.label;
            el.append(sign, name);
            this.flowMarkers.push(new this.maplibregl.Marker({ element: el, anchor: 'left', offset: [-12, 0] })
                .setLngLat(this.#toLngLat(access.x, access.z)).addTo(this.map));
        }
    }

    // Messung als Ebenen von MapLibre: Linien und Punkte in Bildschirmpixeln, damit sie in jeder
    // Zoomstufe gut sichtbar sind; sie liegen über den 3D-Modellen.
    #addMeasureLayers() {
        const color = cssColor('--gelaende-messung', '#dc2626');
        // Aktive Messung (an der gerade gemessen wird) kräftiger, die übrigen etwas dünner
        const active = (on, off) => ['case', ['get', 'active'], on, off];
        this.map.addSource('messung', { type: 'geojson', data: emptyCollection() });
        this.map.addLayer({
            id: 'messung-flaeche', type: 'fill', source: 'messung',
            filter: ['==', '$type', 'Polygon'],
            paint: { 'fill-color': color, 'fill-opacity': active(0.2, 0.12) }
        });
        this.map.addLayer({
            id: 'messung-linie', type: 'line', source: 'messung',
            filter: ['==', '$type', 'LineString'],
            paint: { 'line-color': color, 'line-width': active(3.5, 2.5) }
        });
        this.map.addLayer({
            id: 'messung-punkt', type: 'circle', source: 'messung',
            filter: ['==', '$type', 'Point'],
            paint: {
                'circle-color': color,
                'circle-radius': active(5, 3.5),
                'circle-stroke-color': cssColor('--gelaende-auswahl', '#ffffff'),
                'circle-stroke-width': 1.5
            }
        });
        this.#updateMeasure();
    }

    // Alle Messungen aus dem zuletzt gemeldeten Inhalt zeichnen und beschriften.
    #updateMeasure() {
        for (const m of this.labels ?? []) m.remove();
        this.labels = [];
        const source = this.map?.getSource('messung');
        if (!source) return;
        const features = [];
        for (const measure of this.scene?.measures ?? []) {
            const props = { active: measure.active };
            const coords = measure.points.map(([x, z]) => this.#toLngLat(x, z));
            for (const c of coords) features.push(point(c, props));
            // Umriss immer als Linie (auch wenn er sich überschneidet), die Fläche zusätzlich gefüllt
            if (coords.length >= 2) {
                const line = lineString(measure.closed ? [...coords, coords[0]] : coords);
                line.properties = props;
                features.push(line);
            }
            if (measure.closed) {
                features.push({ type: 'Feature', geometry: { type: 'Polygon', coordinates: [[...coords, coords[0]]] }, properties: props });
            }
            // Beschriftungen als HTML-Marken; Klicks gehen an die Karte durch
            for (const label of measure.labels) {
                const el = document.createElement('div');
                el.className = label.main
                    ? 'badge badge-neutral pointer-events-none font-semibold shadow'
                    : 'badge badge-sm pointer-events-none border-base-300 bg-base-100/90 text-base-content shadow-sm';
                el.textContent = label.text;
                const marker = new this.maplibregl.Marker({ element: el, anchor: label.main ? 'bottom' : 'center' })
                    .setLngLat(this.#toLngLat(label.x, label.z))
                    .addTo(this.map);
                this.labels.push(marker);
            }
        }
        source.setData({ type: 'FeatureCollection', features });
    }

    // Inhalt aus Rust: { drag_elements,  // Elemente lassen sich anfassen, ziehen und drehen
    //                    drag_corners,   // Eckpunkte der Rasenfläche lassen sich ziehen
    //                    elements: [{ kind, x, z, rotation_deg, width_m, depth_m, height_m, outside, selected }],
    //                    corners: [[x, z], …],
    //                    measures: [{ points: [[x, z], …], closed, active, labels: [{ x, z, text, main }] }],
    //                    routes: [{ points, flow, one_way, active, label, label_at }],     (ANF-028)
    //                    accesses: [{ kind, short, label, x, z }],
    //                    zones: [{ points, status, active, label, label_at }] }
    sync(scene) {
        this.scene = scene;
        this.#rebuild();
    }

    // Neuer Ursprung: der ganze Plan wandert mit, die Karte fliegt hin.
    setOrigin(lng, lat) {
        this.origin = [lng, lat];
        if (!this.map) return;
        this.#computeTransform();
        this.#updateOverlays(); // Fläche, Griff und Messung bleiben relativ zum Plan
        this.map.flyTo({ center: this.origin, zoom: Math.max(this.map.getZoom(), 16) });
        this.map.triggerRepaint();
    }

    // Kartenmitte (NaN, solange die Karte nicht geladen ist)
    centerLng() { return this.map ? this.map.getCenter().wrap().lng : NaN; }
    centerLat() { return this.map ? this.map.getCenter().lat : NaN; }

    destroy() {
        this.destroyed = true;
        if (this.endDrag) window.removeEventListener('mouseup', this.endDrag);
        if (this.map) {
            this.map.remove();
            this.map = null;
        }
    }

    // Baut alle 3D-Objekte aus dem zuletzt gemeldeten Inhalt neu auf.
    #rebuild() {
        const THREE = this.THREE;
        if (!this.group || !this.scene) return;

        // Alte Objekte samt Untergruppen entfernen und Speicher freigeben
        for (const child of [...this.group.children]) {
            this.group.remove(child);
            child.traverse((o) => {
                o.geometry?.dispose();
                o.material?.dispose();
            });
        }

        // Elemente als einfache 3D-Modelle; Farben aus dem Farbschema
        const palette = {
            frame: themeColor(THREE, '--gelaende-gestell', 0x374151),
            canvas: themeColor(THREE, '--gelaende-plane', 0xf8fafc)
        };
        const selectColor = themeColor(THREE, '--gelaende-auswahl', 0xffffff);
        this.elementModels = [];
        this.scene.elements.forEach((el, index) => {
            const accent = themeColor(THREE, `--gelaende-${el.kind}`, 0x3b82f6);
            const build = MODELS[el.kind] ?? buildBlock;
            const model = build(THREE, el, { ...palette, accent }, el.outside ? 0.45 : 1.0);
            if (el.selected) {
                // Ausgewählt: leicht aufgehellt und mit Rahmen um die Grundfläche
                model.traverse((o) => o.material?.emissive?.setHex(0x333333));
                const frame = new THREE.LineSegments(
                    new THREE.EdgesGeometry(new THREE.BoxGeometry(el.width_m + 0.4, el.height_m + 0.2, el.depth_m + 0.4)),
                    new THREE.LineBasicMaterial({ color: selectColor })
                );
                frame.position.y = (el.height_m + 0.2) / 2;
                model.add(frame);
            }
            model.userData.index = index;
            model.position.set(el.x, 0, el.z);
            model.rotation.y = (el.rotation_deg * Math.PI) / 180;
            this.group.add(model);
            this.elementModels.push(model);
        });

        // Rasenfläche, Drehgriff und Messung zeichnet MapLibre (siehe #updateOverlays)
        if (this.map) this.#updateOverlays();
        this.map?.triggerRepaint();
    }
}

// ---------------------------------------------------------------------------------------------
// Adresssuche mit dem Suchdienst von swisstopo (geo.admin.ch): Adressen, Orte, Flurnamen, Gemeinden
// und Postleitzahlen der Schweiz und Liechtensteins. Frei nutzbar mit Quellenangabe, ohne Schlüssel.
// Liefert [{ label, lng, lat }] (höchstens fünf Treffer).

const SEARCH_URL = 'https://api3.geo.admin.ch/rest/services/api/SearchServer';

// Die Beschriftungen enthalten Auszeichnungen wie <b>…</b> – als reinen Text lesen (nichts wird ausgeführt)
function plainText(html) {
    return new DOMParser().parseFromString(html ?? '', 'text/html').body.textContent.replace(/\s+/g, ' ').trim();
}

export async function searchAddress(query) {
    const params = new URLSearchParams({
        searchText: query,
        type: 'locations',
        sr: '4326',
        limit: '5',
        lang: 'de'
    });
    let response;
    try {
        response = await fetch(`${SEARCH_URL}?${params}`, { headers: { Accept: 'application/json' } });
    } catch (_) {
        throw new Error('Die Adresssuche ist nicht erreichbar. Bitte Internetverbindung prüfen.');
    }
    if (!response.ok) {
        throw new Error(`Die Adresssuche hat mit Fehler ${response.status} geantwortet. Bitte später erneut versuchen.`);
    }
    const data = await response.json();
    return (data.results ?? [])
        .map((r) => ({
            label: plainText(r.attrs?.label),
            lng: parseFloat(r.attrs?.lon),
            lat: parseFloat(r.attrs?.lat)
        }))
        .filter((r) => r.label && Number.isFinite(r.lng) && Number.isFinite(r.lat));
}

// Lage des Drehgriffs: vor der Vorderseite des Elements (Richtung +z, mitgedreht), 5 m über den Rand
function handlePosition(el) {
    const r = Math.max(el.width_m, el.depth_m) / 2 + 5;
    const a = (el.rotation_deg * Math.PI) / 180;
    return [el.x + Math.sin(a) * r, el.z + Math.cos(a) * r];
}

// Weisser Pfeil nach rechts (Wegrichtung) für die Einbahn-Wege, als Bild für MapLibre
function arrowImage() {
    const size = 32; // bei pixelRatio 2 → 16 px
    const canvas = document.createElement('canvas');
    canvas.width = size;
    canvas.height = size;
    const ctx = canvas.getContext('2d');
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = 5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.beginPath();
    ctx.moveTo(10, 7);
    ctx.lineTo(22, 16);
    ctx.lineTo(10, 25);
    ctx.stroke();
    return ctx.getImageData(0, 0, size, size);
}

// Farbe aus dem Farbschema als CSS-Text (für die Ebenen von MapLibre)
function cssColor(name, fallback) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

function emptyCollection() {
    return { type: 'FeatureCollection', features: [] };
}

function point(coordinates, properties) {
    return { type: 'Feature', geometry: { type: 'Point', coordinates }, properties };
}

function lineString(coordinates) {
    return { type: 'Feature', geometry: { type: 'LineString', coordinates }, properties: {} };
}

// ---------------------------------------------------------------------------------------------
// 3D-Modelle der Elemente. Jedes Modell steht mit dem Mittelpunkt im Ursprung auf dem Boden (y = 0),
// Breite entlang x, Tiefe entlang z, Vorderseite in Richtung +z. Es bleibt innerhalb der Grundfläche
// width_m × depth_m (sie wird in Rust für die Lageprüfung verwendet) und erreicht die Höhe height_m.
// `opacity` < 1: Element steht ausserhalb der Rasenfläche → halbdurchsichtig.

const MODELS = {
    buehne: buildStage,
    verpflegungsstand: buildFoodStall,
    festzelt: buildTent
};

function material(THREE, color, opacity) {
    return new THREE.MeshStandardMaterial({
        color,
        roughness: 0.8,
        transparent: opacity < 1,
        opacity
    });
}

// Quader der Grösse (sx, sy, sz) mit Mittelpunkt (x, y, z)
function box(THREE, group, [sx, sy, sz], [x, y, z], color, opacity) {
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(sx, sy, sz), material(THREE, color, opacity));
    mesh.position.set(x, y, z);
    group.add(mesh);
    return mesh;
}

// Rückfall für unbekannte Arten: einfacher Quader
function buildBlock(THREE, el, colors, opacity) {
    const g = new THREE.Group();
    box(THREE, g, [el.width_m, el.height_m, el.depth_m], [0, el.height_m / 2, 0], colors.accent, opacity);
    return g;
}

// Bühne: Podest, Bühnenboden, Rückwand, vier Stützen, Dach und zwei Lautsprecher
function buildStage(THREE, el, c, opacity) {
    const g = new THREE.Group();
    const w = el.width_m, d = el.depth_m, h = el.height_m;
    const deck = Math.min(1.0, h * 0.25);
    const roof = 0.25;

    box(THREE, g, [w, deck, d], [0, deck / 2, 0], c.accent, opacity); // Podest (Verkleidung)
    box(THREE, g, [w, 0.08, d], [0, deck + 0.04, 0], c.frame, opacity); // Bühnenboden

    const wallH = h - roof - deck;
    box(THREE, g, [w * 0.9, wallH, 0.15], [0, deck + wallH / 2, -d / 2 + 0.35], c.frame, opacity); // Rückwand

    const r = 0.15;
    for (const sx of [-1, 1]) {
        for (const sz of [-1, 1]) {
            const pole = new THREE.Mesh(new THREE.CylinderGeometry(r, r, h - roof, 10), material(THREE, c.frame, opacity));
            pole.position.set(sx * (w / 2 - 0.2), (h - roof) / 2, sz * (d / 2 - 0.2));
            g.add(pole);
        }
        // Lautsprecher vorne links und rechts
        box(THREE, g, [0.8, 1.4, 0.7], [sx * (w / 2 - 0.8), deck + 0.7, d / 2 - 0.7], c.frame, opacity);
    }

    box(THREE, g, [w, roof, d], [0, h - roof / 2, 0], c.accent, opacity); // Dach
    return g;
}

// Verpflegungsstand: Theke, Rück- und Seitenwände, Dach und gestreiftes Vordach
function buildFoodStall(THREE, el, c, opacity) {
    const g = new THREE.Group();
    const w = el.width_m, d = el.depth_m, h = el.height_m;
    const front = Math.min(0.6, d * 0.25); // Platz für das Vordach vor dem Stand
    const bodyD = d - front;
    const bodyZ = -front / 2; // Stand nach hinten versetzt
    const counter = 1.05;
    const roofY = h - 0.35;
    const wall = 0.08;

    box(THREE, g, [w, counter, bodyD], [0, counter / 2, bodyZ], c.accent, opacity); // Thekenkörper
    box(THREE, g, [w, 0.06, bodyD + 0.1], [0, counter + 0.03, bodyZ + 0.05], c.canvas, opacity); // Thekenplatte

    const upper = roofY - counter;
    box(THREE, g, [w, upper, wall], [0, counter + upper / 2, -d / 2 + wall / 2], c.accent, opacity); // Rückwand
    for (const sx of [-1, 1]) {
        box(THREE, g, [wall, upper, bodyD], [sx * (w / 2 - wall / 2), counter + upper / 2, bodyZ], c.accent, opacity);
    }
    box(THREE, g, [w, 0.12, bodyD], [0, roofY + 0.06, bodyZ], c.canvas, opacity); // Dach

    // Vordach: Streifen abwechselnd in der Farbe des Stands und hell, nach vorne geneigt
    const drop = 0.35;
    const slant = Math.hypot(front, drop);
    const awning = new THREE.Group();
    awning.position.set(0, roofY, bodyZ + bodyD / 2);
    awning.rotation.x = Math.atan2(drop, front);
    const stripes = 8;
    for (let i = 0; i < stripes; i++) {
        const sw = w / stripes;
        box(THREE, awning, [sw, 0.04, slant], [-w / 2 + sw * (i + 0.5), 0, slant / 2],
            i % 2 === 0 ? c.accent : c.canvas, opacity);
    }
    g.add(awning);
    return g;
}

// Festzelt: helle Zeltwände, Giebeldach in der Farbe der Art und dunkler Eingang vorne
function buildTent(THREE, el, c, opacity) {
    const g = new THREE.Group();
    const w = el.width_m, d = el.depth_m, h = el.height_m;
    const wallH = h * 0.6;
    const roofH = h - wallH;

    box(THREE, g, [w, wallH, d], [0, wallH / 2, 0], c.canvas, opacity); // Zeltwände

    // Giebeldach: Dreieck (Querschnitt) entlang der Tiefe ausgezogen
    const shape = new THREE.Shape([
        new THREE.Vector2(-w / 2, 0),
        new THREE.Vector2(w / 2, 0),
        new THREE.Vector2(0, roofH)
    ]);
    const roofGeometry = new THREE.ExtrudeGeometry(shape, { depth: d, bevelEnabled: false });
    roofGeometry.translate(0, 0, -d / 2);
    const roof = new THREE.Mesh(roofGeometry, material(THREE, c.accent, opacity));
    roof.position.y = wallH;
    g.add(roof);

    // Eingang vorne
    const doorW = Math.min(1.6, w * 0.4);
    const doorH = Math.min(1.7, wallH * 0.9);
    box(THREE, g, [doorW, doorH, 0.02], [0, doorH / 2, d / 2 + 0.01], c.frame, opacity);
    // Firststange
    box(THREE, g, [0.08, 0.08, d], [0, h, 0], c.frame, opacity);
    return g;
}
