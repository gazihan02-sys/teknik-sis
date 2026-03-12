use leptos::prelude::*;
use std::collections::HashMap;
use shared::dto::service_record_dto::ServiceRecordResponse;
use crate::services::api;

/// Normalize text for search (Turkish-aware lowercase, keeps spaces)
fn normalize_search(s: &str) -> String {
    s.chars().map(|c| match c {
        '\u{0130}' | 'i' => 'i', 'I' | '\u{0131}' => 'i',
        '\u{00d6}' | '\u{00f6}' => 'o', '\u{00dc}' | '\u{00fc}' => 'u',
        '\u{015e}' | '\u{015f}' => 's', '\u{00c7}' | '\u{00e7}' => 'c',
        '\u{011e}' | '\u{011f}' => 'g',
        c => c.to_ascii_lowercase(),
    }).collect()
}

struct StatusCard {
    label: &'static str,
    icon: &'static str,
    status_key: &'static str,
}

const STATUS_CARDS: &[StatusCard] = &[
    StatusCard { label: "M\u{00fc}\u{015f}teri Kabul", icon: "engineering", status_key: "musteri_kabul" },
    StatusCard { label: "Teknisyene Verildi", icon: "schedule", status_key: "teknisyene_verildi" },
    StatusCard { label: "\u{0130}\u{015f}lem Bekliyor", icon: "assignment", status_key: "islem_bekliyor" },
    StatusCard { label: "Par\u{00e7}a Bekliyor", icon: "local_shipping", status_key: "parca_bekliyor" },
    StatusCard { label: "Merkeze Sevk", icon: "sync_alt", status_key: "merkeze_sevk" },
    StatusCard { label: "De\u{011f}i\u{015f}im", icon: "swap_horiz", status_key: "degisim" },
    StatusCard { label: "Tamir Tamamland\u{0131}", icon: "done_all", status_key: "tamir_tamamlandi" },
    StatusCard { label: "Teslim Edildi", icon: "replay", status_key: "teslim_edildi" },
    StatusCard { label: "\u{0130}ade", icon: "fast_rewind", status_key: "iade" },
];

#[component]
pub fn HomePage() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (counts, set_counts) = signal(HashMap::<String, i64>::new());
    let (all_records, set_all_records) = signal(Vec::<ServiceRecordResponse>::new());
    let (records_loaded, set_records_loaded) = signal(false);

    // Fetch counts from API
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get("/api/service-records/counts")
            .send()
            .await
        {
            Ok(resp) if resp.ok() => {
                if let Ok(data) = resp.json::<Vec<shared::dto::service_record_dto::StatusCountResponse>>().await {
                    let map: HashMap<String, i64> = data.into_iter().map(|sc| (sc.status, sc.count)).collect();
                    set_counts.set(map);
                }
            }
            _ => {}
        }
    });

    // Fetch all records for search (lazy — on first keystroke)
    let load_records = move || {
        if !records_loaded.get_untracked() {
            set_records_loaded.set(true);
            leptos::task::spawn_local(async move {
                if let Ok(recs) = api::get_service_records(None).await {
                    set_all_records.set(recs);
                }
            });
        }
    };

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
            _ => "",
        }
    }

    view! {
        <div class="space-y-6">
            // Search bar
            <div class="max-w-2xl mx-auto relative z-30">
                <div class="flex items-center gap-3 bg-surface-container-high rounded-full px-5 py-3 shadow-elevation-1">
                    <span class="material-symbols-outlined text-surface-on-variant" style="font-size:22px">"search"</span>
                    <input
                        type="text"
                        placeholder="Ad Soyad, Telefon veya Fi\u{015f} No ile ara..."
                        class="flex-1 bg-transparent text-body-lg text-surface-on placeholder:text-surface-on-variant focus:outline-none"
                        prop:value=move || search.get()
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            load_records();
                            set_search.set(v);
                        }
                    />
                    {move || {
                        if search.get().is_empty() {
                            view! { <span></span> }.into_any()
                        } else {
                            view! {
                                <button
                                    class="text-surface-on-variant hover:text-surface-on cursor-pointer"
                                    on:click=move |_| set_search.set(String::new())
                                >
                                    <span class="material-symbols-outlined" style="font-size:20px">"close"</span>
                                </button>
                            }.into_any()
                        }
                    }}
                </div>

                // Search results dropdown
                {move || {
                    let q = normalize_search(&search.get());
                    if q.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        let recs = all_records.get();
                        let filtered: Vec<&ServiceRecordResponse> = recs.iter().filter(|r| {
                            normalize_search(&r.customer_name).contains(&q)
                            || normalize_search(&r.phone).contains(&q)
                            || normalize_search(&r.serial_number).contains(&q)
                            || normalize_search(&r.brand).contains(&q)
                            || normalize_search(&r.model).contains(&q)
                        }).take(10).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="absolute left-0 right-0 top-full mt-2 bg-surface-container-lowest rounded-xl shadow-elevation-2 border border-outline-variant/30 p-4 z-50">
                                    <p class="text-body-md text-surface-on-variant text-center">"Sonu\u{00e7} bulunamad\u{0131}"</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="absolute left-0 right-0 top-full mt-2 bg-surface-container-lowest rounded-xl shadow-elevation-2 border border-outline-variant/30 overflow-hidden z-50">
                                    {filtered.into_iter().map(|r| {
                                        let href = format!("/servis-kayitlari/{}", r.status);
                                        let name = r.customer_name.clone();
                                        let phone = r.phone.clone();
                                        let sn = r.serial_number.clone();
                                        let st = status_label(&r.status).to_string();
                                        view! {
                                            <a href=href class="flex items-center justify-between px-4 py-3 hover:bg-surface-container-high transition-colors border-b border-outline-variant/20 last:border-b-0">
                                                <div class="flex flex-col gap-0.5">
                                                    <span class="text-body-md text-surface-on font-medium">{name}</span>
                                                    <span class="text-body-sm text-surface-on-variant">{phone}" · Fi\u{015f} No: "{sn}</span>
                                                </div>
                                                <span class="text-label-sm text-primary bg-primary-container px-2 py-0.5 rounded-full">{st}</span>
                                            </a>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>

            // Status cards grid
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4">
                {STATUS_CARDS.iter().map(|card| {
                    let label = card.label;
                    let icon = card.icon;
                    let status_key = card.status_key.to_string();
                    let href = format!("/servis-kayitlari/{}", card.status_key);
                    view! {
                        <a
                            href=href
                            class="state-layer group relative bg-surface-container-lowest rounded-xl p-4 sm:p-5 shadow-elevation-1 hover:shadow-elevation-2 transition-all duration-200 overflow-hidden flex flex-col justify-between min-h-[130px] border border-outline-variant/30"
                        >
                            // Top row: small icon + label
                            <div class="flex items-center gap-2.5 relative z-10">
                                <div class="w-9 h-9 rounded-full bg-primary-container flex items-center justify-center shrink-0">
                                    <span class="material-symbols-outlined text-primary" style="font-size:18px">{icon}</span>
                                </div>
                                <span class="text-title-md text-surface-on">{label}</span>
                            </div>

                            // Bottom row: count + arrow
                            <div class="flex items-end justify-between relative z-10 mt-3">
                                <span class="text-display-sm text-surface-on font-normal">
                                    {move || counts.get().get(&status_key).copied().unwrap_or(0).to_string()}
                                </span>
                                <span class="material-symbols-outlined text-surface-on-variant group-hover:text-primary transition-colors" style="font-size:20px">"arrow_forward"</span>
                            </div>

                        </a>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
