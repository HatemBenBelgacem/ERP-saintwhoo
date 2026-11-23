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
    // container: HTML-Element der Karte; onClick(x, z) meldet Klicks in Metern;
    // onError(text) meldet, wenn die Karte nicht geladen werden konnte.
    constructor(container, originLng, originLat, onClick, onError) {
        this.origin = [originLng, originLat];
        this.onClick = onClick;
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
                    // Satellitenbilder von Esri World Imagery (Nutzungsbedingungen von Esri beachten)
                    satellite: {
                        type: 'raster',
                        tiles: [
                            'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'
                        ],
                        tileSize: 256,
                        maxzoom: 19,
                        attribution: 'Satellitenbild © Esri, Maxar, Earthstar Geographics'
                    }
                },
                layers: [{ id: 'satellit', type: 'raster', source: 'satellite' }]
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

        map.on('style.load', () => map.addLayer(layer));

        // Klick auf die Karte in Meter relativ zum Ursprung umrechnen und an Rust melden
        map.on('click', (e) => {
            const { x, z } = this.#toLocal(e.lngLat.lng, e.lngLat.lat);
            this.onClick(x, z);
        });
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

    // Inhalt aus Rust: { elements: [{ kind, x, z, rotation_deg, width_m, depth_m, height_m, outside }],
    //                    corners: [[x, z], …] }
    sync(scene) {
        this.scene = scene;
        this.#rebuild();
    }

    // Neuer Ursprung: der ganze Plan wandert mit, die Karte fliegt hin.
    setOrigin(lng, lat) {
        this.origin = [lng, lat];
        if (!this.map) return;
        this.#computeTransform();
        this.map.flyTo({ center: this.origin, zoom: Math.max(this.map.getZoom(), 16) });
        this.map.triggerRepaint();
    }

    // Kartenmitte (NaN, solange die Karte nicht geladen ist)
    centerLng() { return this.map ? this.map.getCenter().wrap().lng : NaN; }
    centerLat() { return this.map ? this.map.getCenter().lat : NaN; }

    destroy() {
        this.destroyed = true;
        if (this.map) {
            this.map.remove();
            this.map = null;
        }
    }

    // Baut alle 3D-Objekte aus dem zuletzt gemeldeten Inhalt neu auf.
    #rebuild() {
        const THREE = this.THREE;
        if (!this.group || !this.scene) return;

        for (const child of [...this.group.children]) {
            this.group.remove(child);
            child.geometry?.dispose();
            child.material?.dispose();
        }

        // Elemente als Quader auf dem Boden
        for (const el of this.scene.elements) {
            const color = themeColor(THREE, `--gelaende-${el.kind}`, 0x3b82f6);
            const geometry = new THREE.BoxGeometry(el.width_m, el.height_m, el.depth_m);
            const material = new THREE.MeshStandardMaterial({
                color,
                transparent: el.outside,
                opacity: el.outside ? 0.45 : 1.0
            });
            const mesh = new THREE.Mesh(geometry, material);
            mesh.position.set(el.x, el.height_m / 2, el.z);
            mesh.rotation.y = (el.rotation_deg * Math.PI) / 180;
            this.group.add(mesh);
        }

        // Rasenfläche: Eckpunkte, Umriss und (ab drei Punkten) halbdurchsichtige Fläche
        const corners = this.scene.corners;
        if (corners.length > 0) {
            const areaColor = themeColor(THREE, '--gelaende-flaeche', 0x22c55e);
            for (const [x, z] of corners) {
                const marker = new THREE.Mesh(
                    new THREE.CylinderGeometry(0.4, 0.4, 1.2, 12),
                    new THREE.MeshStandardMaterial({ color: areaColor })
                );
                marker.position.set(x, 0.6, z);
                this.group.add(marker);
            }
            if (corners.length >= 2) {
                const points = corners.map(([x, z]) => new THREE.Vector3(x, 0.1, z));
                const outline = corners.length >= 3
                    ? new THREE.LineLoop(new THREE.BufferGeometry().setFromPoints(points), new THREE.LineBasicMaterial({ color: areaColor }))
                    : new THREE.Line(new THREE.BufferGeometry().setFromPoints(points), new THREE.LineBasicMaterial({ color: areaColor }));
                this.group.add(outline);
            }
            if (corners.length >= 3) {
                const shape = new THREE.Shape(corners.map(([x, z]) => new THREE.Vector2(x, z)));
                const geometry = new THREE.ShapeGeometry(shape);
                geometry.rotateX(Math.PI / 2); // Form (x, z) aus der xy-Ebene auf den Boden legen
                const fill = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({
                    color: areaColor,
                    transparent: true,
                    opacity: 0.25,
                    side: THREE.DoubleSide,
                    depthWrite: false
                }));
                fill.position.y = 0.05;
                this.group.add(fill);
            }
        }

        this.map?.triggerRepaint();
    }
}
