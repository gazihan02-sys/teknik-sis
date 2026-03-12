use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_location};
use shared::dto::service_record_dto::{CreateServiceRecordRequest, ServiceRecordResponse, UpdateServiceRecordRequest};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::services::api;

/// Convert Turkish characters to ASCII for safe filenames
fn turkish_to_ascii(s: &str) -> String {
    s.chars().map(|c| match c {
        'İ' | 'i' => 'i', 'I' | 'ı' => 'i',
        'Ö' | 'ö' => 'o', 'Ü' | 'ü' => 'u',
        'Ş' | 'ş' => 's', 'Ç' | 'ç' => 'c',
        'Ğ' | 'ğ' => 'g',
        ' ' => '_',
        c if c.is_ascii_alphanumeric() || c == '_' || c == '-' => c.to_ascii_lowercase(),
        c if c.is_alphanumeric() => c.to_ascii_lowercase(),
        _ => '_',
    }).collect()
}

/// Normalize text for search (Turkish-aware lowercase, keeps spaces)
fn normalize_search(s: &str) -> String {
    s.chars().map(|c| match c {
        'İ' | 'i' => 'i', 'I' | 'ı' => 'i',
        'Ö' | 'ö' => 'o', 'Ü' | 'ü' => 'u',
        'Ş' | 'ş' => 's', 'Ç' | 'ç' => 'c',
        'Ğ' | 'ğ' => 'g',
        c => c.to_ascii_lowercase(),
    }).collect()
}

/// Open a base64 image in a new tab via Blob URL
fn open_image(data_url: &str, filename: &str) {
    if data_url.is_empty() { return; }
    let js_fn = js_sys::Reflect::get(
        &web_sys::window().unwrap(),
        &JsValue::from_str("__openBase64Image"),
    ).unwrap();
    let js_fn = js_fn.dyn_into::<js_sys::Function>().unwrap();
    let _ = js_fn.call2(&JsValue::NULL, &JsValue::from_str(data_url), &JsValue::from_str(filename));
}

fn status_label(key: &str) -> &'static str {
    match key {
        "musteri_kabul" => "M\u{00fc}\u{015f}teri Kabul",
        "teknisyene_verildi" => "Teknisyene Verildi",
        "islem_bekliyor" => "\u{0130}\u{015f}lem Bekliyor",
        "parca_bekliyor" => "Par\u{00e7}a Bekliyor",
        "merkeze_sevk" => "Merkeze Sevk",
        "degisim" => "De\u{011f}i\u{015f}im",
        "tamir_tamamlandi" => "Tamir Tamamland\u{0131}",
        "teslim_edildi" => "Teslim Edildi",
        "iade" => "\u{0130}ade",
        _ => "Servis Kay\u{0131}tlar\u{0131}",
    }
}

#[component]
pub fn ServisKayitlariPage() -> impl IntoView {
    let params = use_params_map();
    let status = move || {
        params.read().get("status").unwrap_or_default()
    };

    let (records, set_records) = signal(Vec::<ServiceRecordResponse>::new());
    let (loading, set_loading) = signal(true);
    let (selected_ids, set_selected_ids) = signal(Vec::<String>::new());
    let (search_query, set_search_query) = signal(String::new());
    // Track which record's serial_number is being edited (by id)
    let (editing_serial, set_editing_serial) = signal(Option::<String>::None);
    // Track which record is being edited in the modal (None = create mode)
    let (edit_id, set_edit_id) = signal(Option::<String>::None);
    let location = use_location();
    let (show_form, set_show_form) = signal(false);

    // Reactively open form when ?yeni=true is in URL
    Effect::new(move |_| {
        let search = location.search.get();
        if search.contains("yeni=true") {
            set_show_form.set(true);
        }
    });

    // Form fields
    let (f_ad_soyad, set_f_ad_soyad) = signal(String::new());
    let (f_telefon, set_f_telefon) = signal(String::new());
    let (f_marka_model, set_f_marka_model) = signal(String::new());
    let (f_aksesuarlar, set_f_aksesuarlar) = signal(String::new());
    let (f_musteri_sikayeti, set_f_musteri_sikayeti) = signal(String::new());
    let (f_not, set_f_not) = signal(String::new());
    let (f_seri_no, set_f_seri_no) = signal(String::new());
    let (f_status, set_f_status) = signal(String::new());
    let (form_error, set_form_error) = signal(Option::<String>::None);
    let (saving, set_saving) = signal(false);

    // Document image signals (base64 JPEG, max 300KB)
    let (img_fatura, set_img_fatura) = signal(String::new());
    let (img_garanti, set_img_garanti) = signal(String::new());
    let (img_uretim, set_img_uretim) = signal(String::new());
    let (img_ariza, set_img_ariza) = signal(String::new());

    // Helper: compress file via JS and set signal
    fn handle_file_select(setter: WriteSignal<String>, ev: web_sys::Event) {
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let files = input.files();
        if let Some(file_list) = files {
            if file_list.length() > 0 {
                let file = file_list.get(0).unwrap();
                leptos::task::spawn_local(async move {
                    let window = web_sys::window().unwrap();
                    let compress_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__compressImageToBase64")).unwrap();
                    let compress_fn = compress_fn.dyn_into::<js_sys::Function>().unwrap();
                    let promise = compress_fn.call1(&JsValue::NULL, &file).unwrap();
                    let promise = js_sys::Promise::from(promise);
                    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
                    if let Ok(val) = result {
                        let base64_str = val.as_string().unwrap_or_default();
                        setter.set(base64_str);
                    }
                });
            }
        }
    }

    // Trigger reload
    let (reload_trigger, set_reload_trigger) = signal(0u32);

    // Load records
    let status_for_effect = status.clone();
    Effect::new(move |_| {
        let _trigger = reload_trigger.get();
        let s = status_for_effect();
        set_loading.set(true);
        let s2 = s.clone();
        leptos::task::spawn_local(async move {
            let result = api::get_service_records(Some(&s2)).await;
            match result {
                Ok(data) => set_records.set(data),
                Err(_) => set_records.set(vec![]),
            }
            set_loading.set(false);
        });
    });

    let clear_form = move || {
        set_f_ad_soyad.set(String::new());
        set_f_telefon.set(String::new());
        set_f_marka_model.set(String::new());
        set_f_aksesuarlar.set(String::new());
        set_f_musteri_sikayeti.set(String::new());
        set_f_not.set(String::new());
        set_f_seri_no.set(String::new());
        set_f_status.set(String::new());
        set_img_fatura.set(String::new());
        set_img_garanti.set(String::new());
        set_img_uretim.set(String::new());
        set_img_ariza.set(String::new());
        set_form_error.set(None);
        set_edit_id.set(None);
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_saving.set(true);
        set_form_error.set(None);

        // Parse marka/model from combined field
        let marka_model_str = f_marka_model.get_untracked();
        let parts: Vec<&str> = marka_model_str.splitn(2, ' ').collect();
        let brand = parts.get(0).unwrap_or(&"").to_string();
        let model = parts.get(1).unwrap_or(&"").to_string();

        if let Some(id) = edit_id.get_untracked() {
            // Edit mode — update existing record
            let req = UpdateServiceRecordRequest {
                customer_name: Some(f_ad_soyad.get_untracked()),
                phone: Some(f_telefon.get_untracked()),
                device: None,
                brand: Some(brand),
                model: Some(model),
                serial_number: Some(f_seri_no.get_untracked()),
                issue: Some(f_musteri_sikayeti.get_untracked()),
                notes: Some(f_not.get_untracked()),
                accessories: Some(f_aksesuarlar.get_untracked()),
                status: { let s = f_status.get_untracked(); if s.is_empty() { None } else { Some(s) } },
                doc_fatura: Some(img_fatura.get_untracked()),
                doc_garanti: Some(img_garanti.get_untracked()),
                doc_uretim: Some(img_uretim.get_untracked()),
                doc_ariza: Some(img_ariza.get_untracked()),
            };

            leptos::task::spawn_local(async move {
                match api::update_service_record(&id, req).await {
                    Ok(_) => {
                        clear_form();
                        set_show_form.set(false);
                        set_reload_trigger.update(|v| *v += 1);
                    }
                    Err(e) => set_form_error.set(Some(e)),
                }
                set_saving.set(false);
            });
        } else {
            // Create mode — new record
            let req = CreateServiceRecordRequest {
                customer_name: f_ad_soyad.get_untracked(),
                phone: f_telefon.get_untracked(),
                device: String::new(),
                brand,
                model,
                serial_number: String::new(),
                issue: f_musteri_sikayeti.get_untracked(),
                notes: f_not.get_untracked(),
                accessories: f_aksesuarlar.get_untracked(),
                doc_fatura: img_fatura.get_untracked(),
                doc_garanti: img_garanti.get_untracked(),
                doc_uretim: img_uretim.get_untracked(),
                doc_ariza: img_ariza.get_untracked(),
            };

            leptos::task::spawn_local(async move {
                match api::create_service_record(req).await {
                    Ok(_) => {
                        clear_form();
                        set_show_form.set(false);
                        set_reload_trigger.update(|v| *v += 1);
                    }
                    Err(e) => set_form_error.set(Some(e)),
                }
                set_saving.set(false);
            });
        }
    };

    let on_delete = move |id: String| {
        leptos::task::spawn_local(async move {
            let _ = api::delete_service_record(&id).await;
            set_reload_trigger.update(|v| *v += 1);
        });
    };

    view! {
        <div class="space-y-4">
            // Header row
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                <div class="flex items-center gap-3">
                    <a href="/" class="state-layer w-10 h-10 rounded-full flex items-center justify-center text-surface-on-variant hover:text-surface-on transition-colors">
                        <span class="material-symbols-outlined" style="font-size:22px">"arrow_back"</span>
                    </a>
                    <h1 class="text-headline-sm sm:text-headline-md text-surface-on">
                        {move || status_label(&status())}
                    </h1>
                </div>
            </div>

            // Search bar
            <div class="relative">
                <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-surface-on-variant" style="font-size:20px">"search"</span>
                <input
                    type="text"
                    placeholder="Ad Soyad, Telefon veya Fi\u{015f} No ile ara..."
                    class="w-full pl-10 pr-10 py-2.5 rounded-xl border border-outline-variant bg-surface text-body-md text-surface-on placeholder:text-surface-on-variant/50 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-colors"
                    prop:value=move || search_query.get()
                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                />
                {move || {
                    let q = search_query.get();
                    if q.is_empty() {
                        view! { <span></span> }.into_any()
                    } else {
                        view! {
                            <button
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-surface-on-variant hover:text-surface-on cursor-pointer"
                                on:click=move |_| set_search_query.set(String::new())
                            >
                                <span class="material-symbols-outlined" style="font-size:20px">"close"</span>
                            </button>
                        }.into_any()
                    }
                }}
            </div>

            // Table
            <div class="bg-surface-container-lowest rounded-xl shadow-elevation-1 border border-outline-variant/30 overflow-hidden">
                // Loading state
                {move || {
                    if loading.get() {
                        view! {
                            <div class="flex items-center justify-center py-12">
                                <span class="material-symbols-outlined text-primary animate-spin" style="font-size:32px">"progress_activity"</span>
                            </div>
                        }.into_any()
                    } else {
                        let all = records.get();
                        let q = normalize_search(&search_query.get());
                        let filtered: Vec<ServiceRecordResponse> = if q.is_empty() {
                            all
                        } else {
                            all.into_iter().filter(|r| {
                                normalize_search(&r.customer_name).contains(&q)
                                || normalize_search(&r.phone).contains(&q)
                                || normalize_search(&r.serial_number).contains(&q)
                                || normalize_search(&r.brand).contains(&q)
                                || normalize_search(&r.model).contains(&q)
                            }).collect()
                        };

                        if filtered.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-16 gap-3">
                                    <span class="material-symbols-outlined text-surface-on-variant" style="font-size:48px">"inbox"</span>
                                    <p class="text-body-lg text-surface-on-variant">"Kay\u{0131}t bulunamad\u{0131}"</p>
                                </div>
                            }.into_any()
                        } else {
                            // Desktop table
                            view! {
                                // Desktop
                                <div class="hidden md:block overflow-x-auto">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="border-b border-outline-variant bg-error-container/30">
                                                <th class="w-12 px-4 py-3">
                                                    <input type="checkbox"
                                                        class="w-4 h-4 rounded border-outline-variant accent-primary cursor-pointer"
                                                        prop:checked=move || {
                                                            let sel = selected_ids.get();
                                                            let recs = records.get();
                                                            !recs.is_empty() && sel.len() == recs.len()
                                                        }
                                                        on:change=move |_| {
                                                            let recs = records.get();
                                                            let sel = selected_ids.get();
                                                            if sel.len() == recs.len() {
                                                                set_selected_ids.set(vec![]);
                                                            } else {
                                                                set_selected_ids.set(recs.iter().map(|r| r.id.clone()).collect());
                                                            }
                                                        }
                                                    />
                                                </th>
                                                <th class="text-left text-label-lg text-surface-on-variant px-4 py-3">"İsim"</th>
                                                <th class="text-left text-label-lg text-surface-on-variant px-4 py-3">"Telefon"</th>
                                                <th class="text-left text-label-lg text-surface-on-variant px-4 py-3">"Cihaz Modeli"</th>
                                                <th class="text-left text-label-lg text-surface-on-variant px-4 py-3">"Belgeler"</th>
                                                <th class="text-center text-label-lg text-surface-on-variant px-4 py-3">"İşlemler"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {filtered.iter().map(|r| {
                                                let id = r.id.clone();
                                                let customer = r.customer_name.clone();
                                                let phone = r.phone.clone();
                                                let brand_model = format!("{} {}", r.brand, r.model);
                                                let serial = r.serial_number.clone();
                                                let raw = r.created_at.chars().take(16).collect::<String>();
                                                let date = {
                                                    let sep = if raw.contains('T') { 'T' } else { ' ' };
                                                    let parts: Vec<&str> = raw.splitn(2, sep).collect();
                                                    if parts.len() == 2 {
                                                        let d: Vec<&str> = parts[0].split('-').collect();
                                                        if d.len() == 3 { format!("{}.{}.{} {}", d[2], d[1], d[0], parts[1]) } else { raw.clone() }
                                                    } else { raw.clone() }
                                                };
                                                let has_fatura = !r.doc_fatura.is_empty();
                                                let has_garanti = !r.doc_garanti.is_empty();
                                                let has_uretim = !r.doc_uretim.is_empty();
                                                let has_ariza = !r.doc_ariza.is_empty();
                                                let id_del = id.clone();
                                                let on_delete = on_delete.clone();

                                                // Clones for edit button
                                                let id_edit = id.clone();
                                                let edit_customer = r.customer_name.clone();
                                                let edit_phone = r.phone.clone();
                                                let edit_brand_model = format!("{} {}", r.brand, r.model);
                                                let edit_serial = r.serial_number.clone();
                                                let edit_issue = r.issue.clone();
                                                let edit_notes = r.notes.clone();
                                                let edit_accessories = r.accessories.clone();
                                                let edit_fatura = r.doc_fatura.clone();
                                                let edit_garanti = r.doc_garanti.clone();
                                                let edit_uretim = r.doc_uretim.clone();
                                                let edit_ariza = r.doc_ariza.clone();
                                                let edit_status = r.status.clone();

                                                // Build download links for docs
                                                let fatura_data = r.doc_fatura.clone();
                                                let garanti_data = r.doc_garanti.clone();
                                                let uretim_data = r.doc_uretim.clone();
                                                let ariza_data = r.doc_ariza.clone();
                                                let name_for_file = turkish_to_ascii(&r.customer_name);
                                                let fname_fatura = format!("{}_fatura.jpg", name_for_file);
                                                let fname_garanti = format!("{}_garanti.jpg", name_for_file);
                                                let fname_uretim = format!("{}_uretim.jpg", name_for_file);
                                                let fname_ariza = format!("{}_ariza.jpg", name_for_file);

                                                view! {
                                                    <tr class="border-b border-outline-variant/50 hover:bg-surface-container-low transition-colors">
                                                        <td class="w-12 px-4 py-3">
                                                            <input type="checkbox"
                                                                class="w-4 h-4 rounded border-outline-variant accent-primary cursor-pointer"
                                                                prop:checked={
                                                                    let id = id.clone();
                                                                    move || selected_ids.get().contains(&id)
                                                                }
                                                                on:change={
                                                                    let id = id.clone();
                                                                    move |_| {
                                                                        let mut sel = selected_ids.get();
                                                                        if let Some(pos) = sel.iter().position(|x| x == &id) {
                                                                            sel.remove(pos);
                                                                        } else {
                                                                            sel.push(id.clone());
                                                                        }
                                                                        set_selected_ids.set(sel);
                                                                    }
                                                                }
                                                            />
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <p class="text-body-md text-surface-on font-medium">{customer}</p>
                                                            <p class="text-body-sm text-surface-on-variant">{date}</p>
                                                        </td>
                                                        <td class="px-4 py-3 text-body-md text-surface-on-variant">{phone}</td>
                                                        <td class="px-4 py-3 text-body-md text-surface-on cursor-pointer" title="Çift tıkla: Fiş No düzenle"
                                                            on:dblclick={
                                                                let id = id.clone();
                                                                move |_| set_editing_serial.set(Some(id.clone()))
                                                            }
                                                        >
                                                            {
                                                                let id = id.clone();
                                                                let serial_display = serial.clone();
                                                                let serial_val = serial.clone();
                                                                move || {
                                                                    let editing = editing_serial.get();
                                                                    if editing.as_deref() == Some(&id) {
                                                                        let id_save = id.clone();
                                                                        let id_blur = id.clone();
                                                                        view! {
                                                                            <input
                                                                                type="text"
                                                                                class="w-full px-2 py-1 rounded-lg border border-primary bg-surface text-body-md text-surface-on focus:outline-none focus:ring-2 focus:ring-primary/50"
                                                                                prop:value=serial_val.clone()
                                                                                placeholder="Fi\u{015f} No girin..."
                                                                                on:keydown={
                                                                                    let id_k = id_save.clone();
                                                                                    move |ev: web_sys::KeyboardEvent| {
                                                                                        if ev.key() == "Enter" {
                                                                                            let val = event_target_value(&ev);
                                                                                            let id_c = id_k.clone();
                                                                                            set_editing_serial.set(None);
                                                                                            leptos::task::spawn_local(async move {
                                                                                                let req = UpdateServiceRecordRequest {
                                                                                                    customer_name: None, phone: None, device: None,
                                                                                                    brand: None, model: None,
                                                                                                    serial_number: Some(val),
                                                                                                    issue: None, notes: None, accessories: None,
                                                                                                    status: None, doc_fatura: None, doc_garanti: None,
                                                                                                    doc_uretim: None, doc_ariza: None,
                                                                                                };
                                                                                                let _ = api::update_service_record(&id_c, req).await;
                                                                                                set_reload_trigger.update(|v| *v += 1);
                                                                                            });
                                                                                        } else if ev.key() == "Escape" {
                                                                                            set_editing_serial.set(None);
                                                                                        }
                                                                                    }
                                                                                }
                                                                                on:blur={
                                                                                    let id_b = id_blur.clone();
                                                                                    move |ev: web_sys::FocusEvent| {
                                                                                        let val = event_target_value(&ev);
                                                                                        let id_c = id_b.clone();
                                                                                        set_editing_serial.set(None);
                                                                                        leptos::task::spawn_local(async move {
                                                                                            let req = UpdateServiceRecordRequest {
                                                                                                customer_name: None, phone: None, device: None,
                                                                                                brand: None, model: None,
                                                                                                serial_number: Some(val),
                                                                                                issue: None, notes: None, accessories: None,
                                                                                                status: None, doc_fatura: None, doc_garanti: None,
                                                                                                doc_uretim: None, doc_ariza: None,
                                                                                            };
                                                                                            let _ = api::update_service_record(&id_c, req).await;
                                                                                            set_reload_trigger.update(|v| *v += 1);
                                                                                        });
                                                                                    }
                                                                                }
                                                                            />
                                                                        }.into_any()
                                                                    } else {
                                                                        let bm = brand_model.clone();
                                                                        let sn = serial_display.clone();
                                                                        view! {
                                                                            <div>
                                                                                <p>{bm}</p>
                                                                                {if !sn.is_empty() {
                                                                                    view! { <p class="text-body-sm text-error font-medium">{sn}</p> }.into_any()
                                                                                } else {
                                                                                    view! { <p class="text-body-sm text-surface-on-variant/50 italic">"Fiş No ekle..."</p> }.into_any()
                                                                                }}
                                                                            </div>
                                                                        }.into_any()
                                                                    }
                                                                }
                                                            }
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <div class="flex items-center gap-2">
                                                                // F - Fatura
                                                                {if has_fatura {
                                                                    let data = fatura_data.clone();
                                                                    let fname = fname_fatura.clone();
                                                                    view! {
                                                                        <button
                                                                            on:click=move |_| open_image(&data, &fname)
                                                                            class="w-8 h-8 rounded-lg bg-primary text-white flex items-center justify-center text-label-md font-bold cursor-pointer hover:opacity-80 transition-opacity"
                                                                            title="Fatura - Görüntüle"
                                                                        >"F"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="w-8 h-8 rounded-lg border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-md font-bold">"F"</span>
                                                                    }.into_any()
                                                                }}
                                                                // G - Garanti
                                                                {if has_garanti {
                                                                    let data = garanti_data.clone();
                                                                    let fname = fname_garanti.clone();
                                                                    view! {
                                                                        <button
                                                                            on:click=move |_| open_image(&data, &fname)
                                                                            class="w-8 h-8 rounded-lg bg-primary text-white flex items-center justify-center text-label-md font-bold cursor-pointer hover:opacity-80 transition-opacity"
                                                                            title="Garanti - Görüntüle"
                                                                        >"G"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="w-8 h-8 rounded-lg border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-md font-bold">"G"</span>
                                                                    }.into_any()
                                                                }}
                                                                // Ü - Üretim
                                                                {if has_uretim {
                                                                    let data = uretim_data.clone();
                                                                    let fname = fname_uretim.clone();
                                                                    view! {
                                                                        <button
                                                                            on:click=move |_| open_image(&data, &fname)
                                                                            class="w-8 h-8 rounded-lg bg-primary text-white flex items-center justify-center text-label-md font-bold cursor-pointer hover:opacity-80 transition-opacity"
                                                                            title="Üretim - Görüntüle"
                                                                        >"Ü"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="w-8 h-8 rounded-lg border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-md font-bold">"Ü"</span>
                                                                    }.into_any()
                                                                }}
                                                                // A - Arıza
                                                                {if has_ariza {
                                                                    let data = ariza_data.clone();
                                                                    let fname = fname_ariza.clone();
                                                                    view! {
                                                                        <button
                                                                            on:click=move |_| open_image(&data, &fname)
                                                                            class="w-8 h-8 rounded-lg bg-primary text-white flex items-center justify-center text-label-md font-bold cursor-pointer hover:opacity-80 transition-opacity"
                                                                            title="Arıza - Görüntüle"
                                                                        >"A"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="w-8 h-8 rounded-lg border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-md font-bold">"A"</span>
                                                                    }.into_any()
                                                                }}
                                                            </div>
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <div class="flex items-center justify-center gap-4">
                                                                <button
                                                                    on:click=move |_| {
                                                                        set_f_ad_soyad.set(edit_customer.clone());
                                                                        set_f_telefon.set(edit_phone.clone());
                                                                        set_f_marka_model.set(edit_brand_model.clone());
                                                                        set_f_aksesuarlar.set(edit_accessories.clone());
                                                                        set_f_musteri_sikayeti.set(edit_issue.clone());
                                                                        set_f_not.set(edit_notes.clone());
                                                                        set_f_seri_no.set(edit_serial.clone());
                                                                        set_img_fatura.set(edit_fatura.clone());
                                                                        set_img_garanti.set(edit_garanti.clone());
                                                                        set_img_uretim.set(edit_uretim.clone());
                                                                        set_img_ariza.set(edit_ariza.clone());
                                                                        set_f_status.set(edit_status.clone());
                                                                        set_edit_id.set(Some(id_edit.clone()));
                                                                        set_show_form.set(true);
                                                                    }
                                                                    class="inline-flex items-center justify-center text-primary hover:opacity-70 transition-opacity"
                                                                    title="Düzenle"
                                                                >
                                                                    <span class="material-symbols-outlined" style="font-size:24px">"edit"</span>
                                                                </button>
                                                                <button
                                                                    class="inline-flex items-center justify-center text-green-600 hover:opacity-70 transition-opacity"
                                                                    title="Yazdır"
                                                                >
                                                                    <span class="material-symbols-outlined" style="font-size:24px">"print"</span>
                                                                </button>
                                                                <button
                                                                    on:click=move |_| on_delete(id_del.clone())
                                                                    class="inline-flex items-center justify-center text-error hover:opacity-70 transition-opacity"
                                                                    title="Sil"
                                                                >
                                                                    <span class="material-symbols-outlined" style="font-size:24px">"delete"</span>
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>

                                // Mobile cards
                                <div class="md:hidden divide-y divide-outline-variant/50">
                                    {filtered.iter().map(|r| {
                                        let id = r.id.clone();
                                        let customer = r.customer_name.clone();
                                        let phone = r.phone.clone();
                                        let brand_model = format!("{} {}", r.brand, r.model);
                                        let serial = r.serial_number.clone();
                                        let raw = r.created_at.chars().take(16).collect::<String>();
                                        let date = {
                                            let sep = if raw.contains('T') { 'T' } else { ' ' };
                                            let parts: Vec<&str> = raw.splitn(2, sep).collect();
                                            if parts.len() == 2 {
                                                let d: Vec<&str> = parts[0].split('-').collect();
                                                if d.len() == 3 { format!("{}.{}.{} {}", d[2], d[1], d[0], parts[1]) } else { raw.clone() }
                                            } else { raw.clone() }
                                        };
                                        let has_fatura = !r.doc_fatura.is_empty();
                                        let has_garanti = !r.doc_garanti.is_empty();
                                        let has_uretim = !r.doc_uretim.is_empty();
                                        let has_ariza = !r.doc_ariza.is_empty();
                                        let fatura_data = r.doc_fatura.clone();
                                        let garanti_data = r.doc_garanti.clone();
                                        let uretim_data = r.doc_uretim.clone();
                                        let ariza_data = r.doc_ariza.clone();
                                        let name_for_file = turkish_to_ascii(&r.customer_name);
                                        let fname_fatura = format!("{}_fatura.jpg", name_for_file);
                                        let fname_garanti = format!("{}_garanti.jpg", name_for_file);
                                        let fname_uretim = format!("{}_uretim.jpg", name_for_file);
                                        let fname_ariza = format!("{}_ariza.jpg", name_for_file);
                                        let on_delete = on_delete.clone();

                                        // Clones for edit button
                                        let id_edit = id.clone();
                                        let edit_customer = r.customer_name.clone();
                                        let edit_phone = r.phone.clone();
                                        let edit_brand_model = format!("{} {}", r.brand, r.model);
                                        let edit_serial = r.serial_number.clone();
                                        let edit_issue = r.issue.clone();
                                        let edit_notes = r.notes.clone();
                                        let edit_accessories = r.accessories.clone();
                                        let edit_fatura = r.doc_fatura.clone();
                                        let edit_garanti = r.doc_garanti.clone();
                                        let edit_uretim = r.doc_uretim.clone();
                                        let edit_ariza = r.doc_ariza.clone();
                                        let edit_status = r.status.clone();

                                        view! {
                                            <div class="p-4 space-y-3 flex flex-col items-center text-center">
                                                <div>
                                                    <p class="text-title-md text-surface-on font-medium">{customer}</p>
                                                    <p class="text-body-sm text-surface-on-variant">{phone}" \u{2022} "{date}</p>
                                                </div>
                                                <p class="text-body-md text-surface-on cursor-pointer"
                                                    title="Çift tıkla: Fiş No düzenle"
                                                    on:dblclick={
                                                        let id = id.clone();
                                                        move |_| set_editing_serial.set(Some(id.clone()))
                                                    }
                                                >
                                                    {
                                                        let id = id.clone();
                                                        let serial_display = serial.clone();
                                                        let serial_val = serial.clone();
                                                        move || {
                                                            let editing = editing_serial.get();
                                                            if editing.as_deref() == Some(&id) {
                                                                let id_save = id.clone();
                                                                let id_blur = id.clone();
                                                                view! {
                                                                    <input
                                                                        type="text"
                                                                        class="w-full px-2 py-1 rounded-lg border border-primary bg-surface text-body-md text-surface-on focus:outline-none focus:ring-2 focus:ring-primary/50"
                                                                        prop:value=serial_val.clone()
                                                                        placeholder="Fi\u{015f} No girin..."
                                                                        on:keydown={
                                                                            let id_k = id_save.clone();
                                                                            move |ev: web_sys::KeyboardEvent| {
                                                                                if ev.key() == "Enter" {
                                                                                    let val = event_target_value(&ev);
                                                                                    let id_c = id_k.clone();
                                                                                    set_editing_serial.set(None);
                                                                                    leptos::task::spawn_local(async move {
                                                                                        let req = UpdateServiceRecordRequest {
                                                                                            customer_name: None, phone: None, device: None,
                                                                                            brand: None, model: None,
                                                                                            serial_number: Some(val),
                                                                                            issue: None, notes: None, accessories: None,
                                                                                            status: None, doc_fatura: None, doc_garanti: None,
                                                                                            doc_uretim: None, doc_ariza: None,
                                                                                        };
                                                                                        let _ = api::update_service_record(&id_c, req).await;
                                                                                        set_reload_trigger.update(|v| *v += 1);
                                                                                    });
                                                                                } else if ev.key() == "Escape" {
                                                                                    set_editing_serial.set(None);
                                                                                }
                                                                            }
                                                                        }
                                                                        on:blur={
                                                                            let id_b = id_blur.clone();
                                                                            move |ev: web_sys::FocusEvent| {
                                                                                let val = event_target_value(&ev);
                                                                                let id_c = id_b.clone();
                                                                                set_editing_serial.set(None);
                                                                                leptos::task::spawn_local(async move {
                                                                                    let req = UpdateServiceRecordRequest {
                                                                                        customer_name: None, phone: None, device: None,
                                                                                        brand: None, model: None,
                                                                                        serial_number: Some(val),
                                                                                        issue: None, notes: None, accessories: None,
                                                                                        status: None, doc_fatura: None, doc_garanti: None,
                                                                                        doc_uretim: None, doc_ariza: None,
                                                                                    };
                                                                                    let _ = api::update_service_record(&id_c, req).await;
                                                                                    set_reload_trigger.update(|v| *v += 1);
                                                                                });
                                                                            }
                                                                        }
                                                                    />
                                                                }.into_any()
                                                            } else {
                                                                let bm = brand_model.clone();
                                                                let sn = serial_display.clone();
                                                                view! {
                                                                    <span>
                                                                        {bm}
                                                                        {if !sn.is_empty() {
                                                                            view! { <span class="text-body-sm text-error font-medium">" · "{sn}</span> }.into_any()
                                                                        } else {
                                                                            view! { <span class="text-body-sm text-surface-on-variant/50 italic">" · Fiş No ekle..."</span> }.into_any()
                                                                        }}
                                                                    </span>
                                                                }.into_any()
                                                            }
                                                        }
                                                    }
                                                </p>
                                                // Belgeler
                                                <div class="flex items-center gap-2">
                                                    {if has_fatura {
                                                        let data = fatura_data.clone();
                                                        let fname = fname_fatura.clone();
                                                        view! { <button on:click=move |_| open_image(&data, &fname) class="w-7 h-7 rounded-md bg-primary text-white flex items-center justify-center text-label-sm font-bold">"F"</button> }.into_any()
                                                    } else {
                                                        view! { <span class="w-7 h-7 rounded-md border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-sm font-bold">"F"</span> }.into_any()
                                                    }}
                                                    {if has_garanti {
                                                        let data = garanti_data.clone();
                                                        let fname = fname_garanti.clone();
                                                        view! { <button on:click=move |_| open_image(&data, &fname) class="w-7 h-7 rounded-md bg-primary text-white flex items-center justify-center text-label-sm font-bold">"G"</button> }.into_any()
                                                    } else {
                                                        view! { <span class="w-7 h-7 rounded-md border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-sm font-bold">"G"</span> }.into_any()
                                                    }}
                                                    {if has_uretim {
                                                        let data = uretim_data.clone();
                                                        let fname = fname_uretim.clone();
                                                        view! { <button on:click=move |_| open_image(&data, &fname) class="w-7 h-7 rounded-md bg-primary text-white flex items-center justify-center text-label-sm font-bold">"Ü"</button> }.into_any()
                                                    } else {
                                                        view! { <span class="w-7 h-7 rounded-md border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-sm font-bold">"Ü"</span> }.into_any()
                                                    }}
                                                    {if has_ariza {
                                                        let data = ariza_data.clone();
                                                        let fname = fname_ariza.clone();
                                                        view! { <button on:click=move |_| open_image(&data, &fname) class="w-7 h-7 rounded-md bg-primary text-white flex items-center justify-center text-label-sm font-bold">"A"</button> }.into_any()
                                                    } else {
                                                        view! { <span class="w-7 h-7 rounded-md border-2 border-dashed border-primary/30 text-primary/30 flex items-center justify-center text-label-sm font-bold">"A"</span> }.into_any()
                                                    }}
                                                </div>
                                                // Actions
                                                <div class="flex items-center gap-4">
                                                    <button
                                                        on:click=move |_| {
                                                            set_f_ad_soyad.set(edit_customer.clone());
                                                            set_f_telefon.set(edit_phone.clone());
                                                            set_f_marka_model.set(edit_brand_model.clone());
                                                            set_f_aksesuarlar.set(edit_accessories.clone());
                                                            set_f_musteri_sikayeti.set(edit_issue.clone());
                                                            set_f_not.set(edit_notes.clone());
                                                            set_f_seri_no.set(edit_serial.clone());
                                                            set_img_fatura.set(edit_fatura.clone());
                                                            set_img_garanti.set(edit_garanti.clone());
                                                            set_img_uretim.set(edit_uretim.clone());
                                                            set_img_ariza.set(edit_ariza.clone());
                                                            set_f_status.set(edit_status.clone());
                                                            set_edit_id.set(Some(id_edit.clone()));
                                                            set_show_form.set(true);
                                                        }
                                                        class="inline-flex items-center justify-center text-primary hover:opacity-70 transition-opacity"
                                                    >
                                                        <span class="material-symbols-outlined" style="font-size:22px">"edit"</span>
                                                    </button>
                                                    <button class="inline-flex items-center justify-center text-green-600 hover:opacity-70 transition-opacity">
                                                        <span class="material-symbols-outlined" style="font-size:22px">"print"</span>
                                                    </button>
                                                    <button
                                                        on:click=move |_| on_delete(id.clone())
                                                        class="inline-flex items-center justify-center text-error hover:opacity-70 transition-opacity"
                                                    >
                                                        <span class="material-symbols-outlined" style="font-size:22px">"delete"</span>
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>

            // Record count + batch status change
            {move || {
                let count = records.get().len();
                let sel_count = selected_ids.get().len();
                view! {
                    <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3">
                        <p class="text-body-sm text-surface-on-variant">"Toplam: "{count.to_string()}" kay\u{0131}t"</p>
                        {if sel_count > 0 {
                            view! {
                                <div class="flex items-center gap-2 flex-wrap">
                                    <span class="text-body-sm text-primary font-medium">{sel_count.to_string()}" se\u{00e7}ili \u{2192}"</span>
                                    <select
                                        class="text-body-sm border border-outline-variant rounded-lg px-3 py-1.5 bg-surface text-surface-on focus:outline-none focus:ring-2 focus:ring-primary/50 cursor-pointer"
                                        on:change=move |ev| {
                                            let new_status = event_target_value(&ev);
                                            if new_status.is_empty() { return; }
                                            let ids = selected_ids.get();
                                            let new_status_clone = new_status.clone();
                                            leptos::task::spawn_local(async move {
                                                for id in &ids {
                                                    let req = UpdateServiceRecordRequest {
                                                        customer_name: None,
                                                        phone: None,
                                                        device: None,
                                                        brand: None,
                                                        model: None,
                                                        serial_number: None,
                                                        issue: None,
                                                        notes: None,
                                                        accessories: None,
                                                        status: Some(new_status_clone.clone()),
                                                        doc_fatura: None,
                                                        doc_garanti: None,
                                                        doc_uretim: None,
                                                        doc_ariza: None,
                                                    };
                                                    let _ = api::update_service_record(id, req).await;
                                                }
                                                set_selected_ids.set(vec![]);
                                                set_reload_trigger.update(|v| *v += 1);
                                            });
                                        }
                                    >
                                        <option value="" selected disabled>"Stat\u{00fc} Se\u{00e7}..."</option>
                                        <option value="musteri_kabul">"M\u{00fc}\u{015f}teri Kabul"</option>
                                        <option value="teknisyene_verildi">"Teknisyene Verildi"</option>
                                        <option value="islem_bekliyor">"\u{0130}\u{015f}lem Bekliyor"</option>
                                        <option value="parca_bekliyor">"Par\u{00e7}a Bekliyor"</option>
                                        <option value="merkeze_sevk">"Merkeze Sevk"</option>
                                        <option value="degisim">"De\u{011f}i\u{015f}im"</option>
                                        <option value="tamir_tamamlandi">"Tamir Tamamland\u{0131}"</option>
                                        <option value="teslim_edildi">"Teslim Edildi"</option>
                                        <option value="iade">"\u{0130}ade"</option>
                                    </select>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                }
            }}
        </div>

        // ========== Record Modal (Create / Edit) ==========
        {move || {
            if show_form.get() {
                view! {
                    <div
                        class="fixed inset-0 z-50 flex flex-col md:items-center md:justify-center p-4 bg-black/50 md:overflow-hidden overflow-y-auto"
                        on:click=move |e| {
                            if event_target::<web_sys::HtmlElement>(&e).class_list().contains("fixed") {
                                set_show_form.set(false);
                                set_edit_id.set(None);
                            }
                        }
                    >
                        <div class="bg-surface-container-high rounded-xl shadow-elevation-3 w-full md:max-w-3xl max-h-[90vh] md:max-h-[85vh] overflow-y-auto animate-in my-auto">
                            // Header
                            <div class="flex items-center justify-between px-6 pt-6 pb-3">
                                <h2 class="text-headline-sm text-surface-on">
                                    {move || if edit_id.get().is_some() { "Kay\u{0131}t D\u{00fc}zenle" } else { "Yeni Servis Kayd\u{0131}" }}
                                </h2>
                                <button
                                    on:click=move |_| { set_show_form.set(false); set_edit_id.set(None); }
                                    class="state-layer w-10 h-10 rounded-full flex items-center justify-center text-surface-on-variant hover:text-surface-on transition-colors"
                                >
                                    <span class="material-symbols-outlined" style="font-size:22px">"close"</span>
                                </button>
                            </div>

                            // Form
                            <form on:submit=on_submit class="px-6 pb-6 space-y-4">
                                // Error
                                {move || form_error.get().map(|e| view! {
                                    <div class="bg-error-container text-error-on-container rounded-sm px-4 py-2 text-body-md">{e}</div>
                                })}

                                // Ad Soyad & Telefon
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    <div>
                                        <label class="text-label-md text-surface-on-variant mb-1 block">"Ad Soyad *"</label>
                                        <input
                                            type="text"
                                            required=true
                                            class="md3-input"
                                            prop:value=move || f_ad_soyad.get()
                                            on:input=move |ev| set_f_ad_soyad.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="text-label-md text-surface-on-variant mb-1 block">"Telefon"</label>
                                        <input
                                            type="tel"
                                            class="md3-input"
                                            prop:value=move || f_telefon.get()
                                            on:input=move |ev| set_f_telefon.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>

                                // Marka / Model & Aksesuarlar
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    <div>
                                        <label class="text-label-md text-surface-on-variant mb-1 block">"Marka / Model *"</label>
                                        <input
                                            type="text"
                                            required=true
                                            class="md3-input"
                                            prop:value=move || f_marka_model.get()
                                            on:input=move |ev| set_f_marka_model.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="text-label-md text-surface-on-variant mb-1 block">"Aksesuarlar"</label>
                                        <input
                                            type="text"
                                            class="md3-input"
                                            prop:value=move || f_aksesuarlar.get()
                                            on:input=move |ev| set_f_aksesuarlar.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>

                                // Belgeler (Documents)
                                <div>
                                    <p class="text-label-md text-surface-on-variant mb-1.5">"Belgeler"</p>
                                    <div class="flex gap-2">
                                        // Fatura
                                        <label class="flex-1">
                                            <input
                                                type="file"
                                                class="hidden"
                                                accept="image/*"
                                                on:change=move |ev: web_sys::Event| handle_file_select(set_img_fatura, ev)
                                            />
                                            <div class="state-layer flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border border-dashed cursor-pointer hover:bg-surface-container-high transition-colors"
                                                class:border-outline-variant=move || img_fatura.get().is_empty()
                                                class:border-primary=move || !img_fatura.get().is_empty()
                                                class:bg-primary-container=move || !img_fatura.get().is_empty()
                                            >
                                                <span class="material-symbols-outlined" style="font-size:18px"
                                                    class:text-surface-on-variant=move || img_fatura.get().is_empty()
                                                    class:text-primary=move || !img_fatura.get().is_empty()
                                                >{move || if img_fatura.get().is_empty() { "attach_file" } else { "check_circle" }}</span>
                                                <span class="text-label-sm"
                                                    class:text-surface-on-variant=move || img_fatura.get().is_empty()
                                                    class:text-primary=move || !img_fatura.get().is_empty()
                                                >"Fatura"</span>
                                            </div>
                                        </label>

                                        // Garanti
                                        <label class="flex-1">
                                            <input
                                                type="file"
                                                class="hidden"
                                                accept="image/*"
                                                on:change=move |ev: web_sys::Event| handle_file_select(set_img_garanti, ev)
                                            />
                                            <div class="state-layer flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border border-dashed cursor-pointer hover:bg-surface-container-high transition-colors"
                                                class:border-outline-variant=move || img_garanti.get().is_empty()
                                                class:border-primary=move || !img_garanti.get().is_empty()
                                                class:bg-primary-container=move || !img_garanti.get().is_empty()
                                            >
                                                <span class="material-symbols-outlined" style="font-size:18px"
                                                    class:text-surface-on-variant=move || img_garanti.get().is_empty()
                                                    class:text-primary=move || !img_garanti.get().is_empty()
                                                >{move || if img_garanti.get().is_empty() { "attach_file" } else { "check_circle" }}</span>
                                                <span class="text-label-sm"
                                                    class:text-surface-on-variant=move || img_garanti.get().is_empty()
                                                    class:text-primary=move || !img_garanti.get().is_empty()
                                                >"Garanti"</span>
                                            </div>
                                        </label>

                                        // Üretim
                                        <label class="flex-1">
                                            <input
                                                type="file"
                                                class="hidden"
                                                accept="image/*"
                                                on:change=move |ev: web_sys::Event| handle_file_select(set_img_uretim, ev)
                                            />
                                            <div class="state-layer flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border border-dashed cursor-pointer hover:bg-surface-container-high transition-colors"
                                                class:border-outline-variant=move || img_uretim.get().is_empty()
                                                class:border-primary=move || !img_uretim.get().is_empty()
                                                class:bg-primary-container=move || !img_uretim.get().is_empty()
                                            >
                                                <span class="material-symbols-outlined" style="font-size:18px"
                                                    class:text-surface-on-variant=move || img_uretim.get().is_empty()
                                                    class:text-primary=move || !img_uretim.get().is_empty()
                                                >{move || if img_uretim.get().is_empty() { "attach_file" } else { "check_circle" }}</span>
                                                <span class="text-label-sm"
                                                    class:text-surface-on-variant=move || img_uretim.get().is_empty()
                                                    class:text-primary=move || !img_uretim.get().is_empty()
                                                >"\u{00dc}retim"</span>
                                            </div>
                                        </label>

                                        // Arıza
                                        <label class="flex-1">
                                            <input
                                                type="file"
                                                class="hidden"
                                                accept="image/*"
                                                on:change=move |ev: web_sys::Event| handle_file_select(set_img_ariza, ev)
                                            />
                                            <div class="state-layer flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border border-dashed cursor-pointer hover:bg-surface-container-high transition-colors"
                                                class:border-outline-variant=move || img_ariza.get().is_empty()
                                                class:border-primary=move || !img_ariza.get().is_empty()
                                                class:bg-primary-container=move || !img_ariza.get().is_empty()
                                            >
                                                <span class="material-symbols-outlined" style="font-size:18px"
                                                    class:text-surface-on-variant=move || img_ariza.get().is_empty()
                                                    class:text-primary=move || !img_ariza.get().is_empty()
                                                >{move || if img_ariza.get().is_empty() { "attach_file" } else { "check_circle" }}</span>
                                                <span class="text-label-sm"
                                                    class:text-surface-on-variant=move || img_ariza.get().is_empty()
                                                    class:text-primary=move || !img_ariza.get().is_empty()
                                                >"Ar\u{0131}za"</span>
                                            </div>
                                        </label>
                                    </div>
                                </div>

                                // Müşteri Şikayeti
                                <div>
                                    <label class="text-label-md text-surface-on-variant mb-1 block">"Müşteri Şikayeti *"</label>
                                    <textarea
                                        required=true
                                        rows=3
                                        class="md3-input resize-none"
                                        prop:value=move || f_musteri_sikayeti.get()
                                        on:input=move |ev| set_f_musteri_sikayeti.set(event_target_value(&ev))
                                    ></textarea>
                                </div>

                                // Not (Varsa)
                                {move || {
                                    if edit_id.get().is_some() {
                                        view! {
                                            <div>
                                                <label class="text-label-md text-surface-on-variant mb-1 block">"Teknisyen A\u{00e7}\u{0131}klamas\u{0131}"</label>
                                                <textarea
                                                    rows=3
                                                    class="md3-input resize-none"
                                                    prop:value=move || f_not.get()
                                                    on:input=move |ev| set_f_not.set(event_target_value(&ev))
                                                ></textarea>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div class="hidden"></div> }.into_any()
                                    }
                                }}

                                // Statü (sadece edit modda)
                                {move || {
                                    if edit_id.get().is_some() {
                                        view! {
                                            <div>
                                                <label class="text-label-md text-surface-on-variant mb-1 block">"Stat\u{00fc}"</label>
                                                <select
                                                    class="md3-input"
                                                    on:change=move |ev| set_f_status.set(event_target_value(&ev))
                                                >
                                                    <option value="musteri_kabul" selected=move || f_status.get() == "musteri_kabul">"M\u{00fc}\u{015f}teri Kabul"</option>
                                                    <option value="teknisyene_verildi" selected=move || f_status.get() == "teknisyene_verildi">"Teknisyene Verildi"</option>
                                                    <option value="islem_bekliyor" selected=move || f_status.get() == "islem_bekliyor">"\u{0130}\u{015f}lem Bekliyor"</option>
                                                    <option value="parca_bekliyor" selected=move || f_status.get() == "parca_bekliyor">"Par\u{00e7}a Bekliyor"</option>
                                                    <option value="merkeze_sevk" selected=move || f_status.get() == "merkeze_sevk">"Merkeze Sevk"</option>
                                                    <option value="degisim" selected=move || f_status.get() == "degisim">"De\u{011f}i\u{015f}im"</option>
                                                    <option value="tamir_tamamlandi" selected=move || f_status.get() == "tamir_tamamlandi">"Tamir Tamamland\u{0131}"</option>
                                                    <option value="teslim_edildi" selected=move || f_status.get() == "teslim_edildi">"Teslim Edildi"</option>
                                                    <option value="iade" selected=move || f_status.get() == "iade">"\u{0130}ade"</option>
                                                </select>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div class="hidden"></div> }.into_any()
                                    }
                                }}

                                // Actions
                                <div class="flex flex-nowrap justify-end gap-2 pt-2">
                                    <button
                                        type="button"
                                        on:click=move |_| { set_show_form.set(false); set_edit_id.set(None); }
                                        class="state-layer px-5 h-10 rounded-full text-label-lg text-primary hover:bg-primary-container transition-colors"
                                    >
                                        "\u{0130}ptal"
                                    </button>
                                    <button
                                        type="submit"
                                        disabled=move || saving.get()
                                        class="state-layer inline-flex items-center gap-2 bg-primary text-primary-on px-6 h-10 rounded-full text-label-lg shadow-elevation-1 hover:shadow-elevation-2 transition-all disabled:opacity-60"
                                    >
                                        {move || if saving.get() { "Kaydediliyor..." } else if edit_id.get().is_some() { "G\u{00fc}ncelle" } else { "Kaydet" }}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div class="hidden"></div> }.into_any()
            }
        }}
    }
}
