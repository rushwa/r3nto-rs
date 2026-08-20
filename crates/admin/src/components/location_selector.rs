use dioxus::prelude::*;
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";

#[derive(Clone, Debug, Default,PartialEq)]
pub struct LocationSelection {
    pub country_id: Option<i32>,
    pub county_id: Option<i32>,
    pub constituency_id: Option<i32>,
    pub ward_id: Option<i32>,
    pub location_id: Option<i32>,
    pub village: String,
}

#[component]
pub fn LocationSelector(
    selection: LocationSelection,
    on_change: EventHandler<LocationSelection>,
) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut countries = use_signal(|| Vec::<(i32, String)>::new());
    let mut counties = use_signal(|| Vec::<(i32, String)>::new());
    let mut constituencies = use_signal(|| Vec::<(i32, String)>::new());
    let mut wards = use_signal(|| Vec::<(i32, String)>::new());
    let mut locations = use_signal(|| Vec::<(i32, String)>::new());

    let mut loading_counties = use_signal(|| false);
    let mut loading_constituencies = use_signal(|| false);
    let mut loading_wards = use_signal(|| false);
    let mut loading_locations = use_signal(|| false);

    let current = selection.clone();
    let village_val = current.village.clone();

    // Load countries on mount
    let token_countries = token.clone();
    use_effect(move || {
        let t = token_countries.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/locations/countries", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                    let list: Vec<(i32, String)> = data.into_iter().filter_map(|v| {
                        let id = v.get("id")?.as_i64()? as i32;
                        let name = v.get("name")?.as_str()?.to_string();
                        Some((id, name))
                    }).collect();
                    countries.set(list);
                }
            }
        });
    });

    // Load counties when country changes
    let token_counties = token.clone();
    let country_id = current.country_id;
    use_effect(move || {
        let t = token_counties.clone();
        let cid = country_id;
        let mut loading = loading_counties;
        spawn(async move {
            counties.set(Vec::new());
            constituencies.set(Vec::new());
            wards.set(Vec::new());
            locations.set(Vec::new());

            if let Some(parent_id) = cid {
                loading.set(true);
                if let Ok(resp) = reqwest::Client::new()
                    .get(&format!("{}/admin/locations/{}/children", API_BASE_URL, parent_id))
                    .header("Authorization", format!("Bearer {}", t))
                    .send().await
                {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        let list: Vec<(i32, String)> = data.into_iter().filter_map(|v| {
                            let id = v.get("id")?.as_i64()? as i32;
                            let name = v.get("name")?.as_str()?.to_string();
                            Some((id, name))
                        }).collect();
                        counties.set(list);
                    }
                }
                loading.set(false);
            }
        });
    });

    // Load constituencies when county changes
    let token_const = token.clone();
    let county_id = current.county_id;
    use_effect(move || {
        let t = token_const.clone();
        let cid = county_id;
        let mut loading = loading_constituencies;
        spawn(async move {
            constituencies.set(Vec::new());
            wards.set(Vec::new());
            locations.set(Vec::new());

            if let Some(parent_id) = cid {
                loading.set(true);
                if let Ok(resp) = reqwest::Client::new()
                    .get(&format!("{}/admin/locations/{}/children", API_BASE_URL, parent_id))
                    .header("Authorization", format!("Bearer {}", t))
                    .send().await
                {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        let list: Vec<(i32, String)> = data.into_iter().filter_map(|v| {
                            let id = v.get("id")?.as_i64()? as i32;
                            let name = v.get("name")?.as_str()?.to_string();
                            Some((id, name))
                        }).collect();
                        constituencies.set(list);
                    }
                }
                loading.set(false);
            }
        });
    });

    // Load wards when constituency changes
    let token_wards = token.clone();
    let constituency_id = current.constituency_id;
    use_effect(move || {
        let t = token_wards.clone();
        let cid = constituency_id;
        let mut loading = loading_wards;
        spawn(async move {
            wards.set(Vec::new());
            locations.set(Vec::new());

            if let Some(parent_id) = cid {
                loading.set(true);
                if let Ok(resp) = reqwest::Client::new()
                    .get(&format!("{}/admin/locations/{}/children", API_BASE_URL, parent_id))
                    .header("Authorization", format!("Bearer {}", t))
                    .send().await
                {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        let list: Vec<(i32, String)> = data.into_iter().filter_map(|v| {
                            let id = v.get("id")?.as_i64()? as i32;
                            let name = v.get("name")?.as_str()?.to_string();
                            Some((id, name))
                        }).collect();
                        wards.set(list);
                    }
                }
                loading.set(false);
            }
        });
    });

    // Load locations when ward changes
    let token_locs = token.clone();
    let ward_id = current.ward_id;
    use_effect(move || {
        let t = token_locs.clone();
        let wid = ward_id;
        let mut loading = loading_locations;
        spawn(async move {
            locations.set(Vec::new());

            if let Some(parent_id) = wid {
                loading.set(true);
                if let Ok(resp) = reqwest::Client::new()
                    .get(&format!("{}/admin/locations/{}/children", API_BASE_URL, parent_id))
                    .header("Authorization", format!("Bearer {}", t))
                    .send().await
                {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        let list: Vec<(i32, String)> = data.into_iter().filter_map(|v| {
                            let id = v.get("id")?.as_i64()? as i32;
                            let name = v.get("name")?.as_str()?.to_string();
                            Some((id, name))
                        }).collect();
                        locations.set(list);
                    }
                }
                loading.set(false);
            }
        });
    });

    let on_country_change = {
        let current = current.clone();
        let on_change = on_change.clone();
        move |evt: Event<FormData>| {
            let val: Option<i32> = evt.value().parse().ok();
            let mut new_sel = current.clone();
            new_sel.country_id = val;
            new_sel.county_id = None;
            new_sel.constituency_id = None;
            new_sel.ward_id = None;
            new_sel.location_id = None;
            on_change.call(new_sel);
        }
    };

    let on_county_change = {
        let current = current.clone();
        let on_change = on_change.clone();
        move |evt: Event<FormData>| {
            let val: Option<i32> = evt.value().parse().ok();
            let mut new_sel = current.clone();
            new_sel.county_id = val;
            new_sel.constituency_id = None;
            new_sel.ward_id = None;
            new_sel.location_id = None;
            on_change.call(new_sel);
        }
    };

    let on_constituency_change = {
        let current = current.clone();
        let on_change = on_change.clone();
        move |evt: Event<FormData>| {
            let val: Option<i32> = evt.value().parse().ok();
            let mut new_sel = current.clone();
            new_sel.constituency_id = val;
            new_sel.ward_id = None;
            new_sel.location_id = None;
            on_change.call(new_sel);
        }
    };

    let on_ward_change = {
        let current = current.clone();
        let on_change = on_change.clone();
        move |evt: Event<FormData>| {
            let val: Option<i32> = evt.value().parse().ok();
            let mut new_sel = current.clone();
            new_sel.ward_id = val;
            new_sel.location_id = None;
            on_change.call(new_sel);
        }
    };

    let on_location_change = {
        let current = current.clone();
        let on_change = on_change.clone();
        move |evt: Event<FormData>| {
            let val: Option<i32> = evt.value().parse().ok();
            let mut new_sel = current.clone();
            new_sel.location_id = val;
            on_change.call(new_sel);
        }
    };

    let on_village_change = {
        let current = current.clone();
        let on_change = on_change.clone();
        move |evt: Event<FormData>| {
            let mut new_sel = current.clone();
            new_sel.village = evt.value();
            on_change.call(new_sel);
        }
    };

    rsx! {
        div { class: "space-y-4",
            // Country (default Kenya)
            div {
                label { class: "block text-gray-400 text-sm mb-1", "Country *" }
                select {
                    class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                    value: current.country_id.map(|i| i.to_string()).unwrap_or_default(),
                    onchange: on_country_change,
                    option { value: "", "Select Country" }
                    for (id, name) in countries.read().iter() {
                        option { value: "{id}", "{name}" }
                    }
                }
            }

            // County
            div {
                label { class: "block text-gray-400 text-sm mb-1", "County *" }
                if *loading_counties.read() {
                    div { class: "text-gray-500 text-sm", "Loading counties..." }
                } else {
                    select {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg disabled:opacity-50",
                        disabled: current.country_id.is_none(),
                        value: current.county_id.map(|i| i.to_string()).unwrap_or_default(),
                        onchange: on_county_change,
                        option { value: "", "Select County" }
                        for (id, name) in counties.read().iter() {
                            option { value: "{id}", "{name}" }
                        }
                    }
                }
            }

            // Constituency
            div {
                label { class: "block text-gray-400 text-sm mb-1", "Constituency *" }
                if *loading_constituencies.read() {
                    div { class: "text-gray-500 text-sm", "Loading constituencies..." }
                } else {
                    select {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg disabled:opacity-50",
                        disabled: current.county_id.is_none(),
                        value: current.constituency_id.map(|i| i.to_string()).unwrap_or_default(),
                        onchange: on_constituency_change,
                        option { value: "", "Select Constituency" }
                        for (id, name) in constituencies.read().iter() {
                            option { value: "{id}", "{name}" }
                        }
                    }
                }
            }

            // Ward
            div {
                label { class: "block text-gray-400 text-sm mb-1", "Ward *" }
                if *loading_wards.read() {
                    div { class: "text-gray-500 text-sm", "Loading wards..." }
                } else {
                    select {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg disabled:opacity-50",
                        disabled: current.constituency_id.is_none(),
                        value: current.ward_id.map(|i| i.to_string()).unwrap_or_default(),
                        onchange: on_ward_change,
                        option { value: "", "Select Ward" }
                        for (id, name) in wards.read().iter() {
                            option { value: "{id}", "{name}" }
                        }
                    }
                }
            }

            // Location
            div {
                label { class: "block text-gray-400 text-sm mb-1", "Location" }
                if *loading_locations.read() {
                    div { class: "text-gray-500 text-sm", "Loading locations..." }
                } else if locations.read().is_empty() && current.ward_id.is_some() {
                    div { class: "text-gray-500 text-sm italic", "No sub-locations available" }
                } else {
                    select {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg disabled:opacity-50",
                        disabled: current.ward_id.is_none(),
                        value: current.location_id.map(|i| i.to_string()).unwrap_or_default(),
                        onchange: on_location_change,
                        option { value: "", "Select Location (optional)" }
                        for (id, name) in locations.read().iter() {
                            option { value: "{id}", "{name}" }
                        }
                    }
                }
            }

            // Village (free text)
            div {
                label { class: "block text-gray-400 text-sm mb-1", "Village / Estate" }
                input {
                    class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                    placeholder: "e.g., Kileleshwa, Lavington, Kilimani",
                    value: "{village_val}",
                    oninput: on_village_change,
                }
            }
        }
    }
}