use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function initMap(containerId, lat, lng, onLocationSelect) {
    const container = document.getElementById(containerId);
    if (!container) return;

    // Use Leaflet via CDN (add to your index.html: <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>)
    if (typeof L === 'undefined') {
        container.innerHTML = '<p class="text-red-400 p-4">Map library not loaded. Add Leaflet CDN to index.html</p>';
        return null;
    }

    const map = L.map(containerId).setView([lat || -1.2921, lng || 36.8219], 13);

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '© OpenStreetMap contributors'
    }).addTo(map);

    let marker = null;
    if (lat && lng) {
        marker = L.marker([lat, lng]).addTo(map);
    }

    map.on('click', function(e) {
        if (marker) map.removeLayer(marker);
        marker = L.marker(e.latlng).addTo(map);

        // Reverse geocode
        fetch(`https://nominatim.openstreetmap.org/reverse?format=json&lat=${e.latlng.lat}&lon=${e.latlng.lng}`)
            .then(r => r.json())
            .then(data => {
                const address = data.display_name || `${e.latlng.lat.toFixed(6)}, ${e.latlng.lng.toFixed(6)}`;
                onLocationSelect(e.latlng.lat, e.latlng.lng, address);
            })
            .catch(() => {
                onLocationSelect(e.latlng.lat, e.latlng.lng, `${e.latlng.lat.toFixed(6)}, ${e.latlng.lng.toFixed(6)}`);
            });
    });

    return map;
}

export function updateMapMarker(map, lat, lng) {
    // Marker update handled by re-init
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = initMap)]
    fn init_map_js(container_id: &str, lat: f64, lng: f64, callback: &js_sys::Function) -> JsValue;
}

#[derive(Clone, Debug, Default,PartialEq)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub map_address: String,
}

#[component]
pub fn MapPicker(
    location: GeoLocation,
    on_change: EventHandler<GeoLocation>,
) -> Element {
    let container_id = "property-map-picker";

    let callback = Closure::wrap(Box::new(move |lat: f64, lng: f64, address: String| {
        // This would be called by JS
    }) as Box<dyn FnMut(f64, f64, String)>);

    rsx! {
        div { class: "space-y-3",
            label { class: "block text-gray-400 text-sm mb-1", "📍 Pin Location on Map" }
            p { class: "text-gray-500 text-xs mb-2",
                "Click on the map to set the exact property location. This helps clients find your property."
            }
            div {
                id: container_id,
                class: "w-full h-64 bg-gray-700 rounded-lg border border-gray-600 relative overflow-hidden",
                // Fallback when Leaflet isn't loaded
                div { class: "absolute inset-0 flex items-center justify-center",
                    div { class: "text-center",
                        p { class: "text-gray-400 text-sm", "Interactive Map" }
                        p { class: "text-gray-500 text-xs mt-1", "Click to set location" }
                    }
                }
            }

            // Manual coordinate input
            div { class: "grid grid-cols-2 gap-3",
                div {
                    label { class: "block text-gray-400 text-xs mb-1", "Latitude" }
                    input {
                        class: "w-full px-2 py-1.5 bg-gray-700 text-white border border-gray-600 rounded text-sm",
                        r#type: "number",
                        step: "0.000001",
                        placeholder: "-1.2921",
                        value: if location.latitude != 0.0 { format!("{:.6}", location.latitude) } else { String::new() },
                    }
                }
                div {
                    label { class: "block text-gray-400 text-xs mb-1", "Longitude" }
                    input {
                        class: "w-full px-2 py-1.5 bg-gray-700 text-white border border-gray-600 rounded text-sm",
                        r#type: "number",
                        step: "0.000001",
                        placeholder: "36.8219",
                        value: if location.longitude != 0.0 { format!("{:.6}", location.longitude) } else { String::new() },
                    }
                }
            }

            if !location.map_address.is_empty() {
                div { class: "bg-gray-800 border border-gray-700 rounded-lg p-3",
                    p { class: "text-gray-400 text-xs", "📍 Address:" }
                    p { class: "text-white text-sm mt-1", "{location.map_address}" }
                }
            }
        }
    }
}