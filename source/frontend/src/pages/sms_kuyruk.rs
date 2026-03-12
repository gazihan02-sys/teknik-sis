use leptos::prelude::*;
use leptos::task;
use crate::services::api;
use shared::dto::sms_log_dto::SmsLogResponse;

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
        _ => "Bilinmeyen",
    }
}

fn status_color(key: &str) -> &'static str {
    match key {
        "musteri_kabul" => "bg-blue-100 text-blue-800",
        "teknisyene_verildi" => "bg-indigo-100 text-indigo-800",
        "islem_bekliyor" => "bg-amber-100 text-amber-800",
        "parca_bekliyor" => "bg-orange-100 text-orange-800",
        "merkeze_sevk" => "bg-purple-100 text-purple-800",
        "degisim" => "bg-pink-100 text-pink-800",
        "tamir_tamamlandi" => "bg-green-100 text-green-800",
        "teslim_edildi" => "bg-teal-100 text-teal-800",
        "iade" => "bg-red-100 text-red-800",
        _ => "bg-gray-100 text-gray-800",
    }
}

#[component]
pub fn SmsKuyrukPage() -> impl IntoView {
    let (logs, set_logs) = signal(Vec::<SmsLogResponse>::new());
    let (loading, set_loading) = signal(true);

    Effect::new(move |_| {
        task::spawn_local(async move {
            match api::get_sms_logs().await {
                Ok(data) => set_logs.set(data),
                Err(_) => set_logs.set(vec![]),
            }
            set_loading.set(false);
        });
    });

    view! {
        <div class="flex-1 flex flex-col gap-4 p-4 sm:p-6 max-w-5xl mx-auto w-full">
            // Header
            <div class="flex items-center gap-3">
                <a href="/settings" class="inline-flex items-center justify-center w-10 h-10 rounded-full hover:bg-surface-container-highest transition-colors">
                    <span class="material-symbols-outlined" style="font-size:24px">"arrow_back"</span>
                </a>
                <div>
                    <h1 class="text-headline-sm text-surface-on font-bold">"SMS Kuyru\u{011f}u"</h1>
                    <p class="text-body-sm text-surface-on-variant">"G\u{00f6}nderilen t\u{00fc}m SMS bildirimleri"</p>
                </div>
            </div>

            // Content
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex items-center justify-center py-16">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                        </div>
                    }.into_any()
                } else {
                    let items = logs.get();
                    if items.is_empty() {
                        view! {
                            <div class="flex flex-col items-center justify-center py-16 gap-3">
                                <span class="material-symbols-outlined text-surface-on-variant" style="font-size:48px">"sms"</span>
                                <p class="text-body-lg text-surface-on-variant">"Hen\u{00fc}z SMS g\u{00f6}nderilmemi\u{015f}"</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-col gap-2">
                                // Toplam
                                <p class="text-body-sm text-surface-on-variant">"Toplam: "{items.len().to_string()}" SMS"</p>

                                // Desktop table
                                <div class="hidden md:block bg-surface-container-low rounded-xl shadow-elevation-1 overflow-hidden">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="border-b border-outline-variant">
                                                <th class="px-4 py-3 text-left text-label-md text-surface-on-variant font-medium">"Tarih"</th>
                                                <th class="px-4 py-3 text-left text-label-md text-surface-on-variant font-medium">"M\u{00fc}\u{015f}teri"</th>
                                                <th class="px-4 py-3 text-left text-label-md text-surface-on-variant font-medium">"Telefon"</th>
                                                <th class="px-4 py-3 text-center text-label-md text-surface-on-variant font-medium">"Stat\u{00fc}"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.iter().map(|log| {
                                                let date = if log.created_at.len() >= 16 { &log.created_at[..16] } else { &log.created_at };
                                                let color = status_color(&log.status);
                                                let label = status_label(&log.status);
                                                view! {
                                                    <tr class="border-b border-outline-variant/50 hover:bg-surface-container transition-colors">
                                                        <td class="px-4 py-3 text-body-sm text-surface-on-variant whitespace-nowrap">{date.to_string()}</td>
                                                        <td class="px-4 py-3 text-body-sm text-surface-on font-medium">{log.customer_name.clone()}</td>
                                                        <td class="px-4 py-3 text-body-sm text-surface-on-variant">{log.phone.clone()}</td>
                                                        <td class="px-4 py-3 text-center">
                                                            <span class={format!("inline-block px-2.5 py-0.5 rounded-full text-label-sm font-medium {}", color)}>
                                                                {label}
                                                            </span>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>

                                // Mobile cards
                                <div class="md:hidden flex flex-col gap-2">
                                    {items.iter().map(|log| {
                                        let date = if log.created_at.len() >= 16 { &log.created_at[..16] } else { &log.created_at };
                                        let color = status_color(&log.status);
                                        let label = status_label(&log.status);
                                        view! {
                                            <div class="bg-surface-container-low rounded-xl p-4 shadow-elevation-1 space-y-2">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-body-sm text-surface-on font-medium">{log.customer_name.clone()}</span>
                                                    <span class={format!("px-2 py-0.5 rounded-full text-label-sm font-medium {}", color)}>
                                                        {label}
                                                    </span>
                                                </div>
                                                <div class="flex items-center gap-2 text-body-sm text-surface-on-variant">
                                                    <span class="material-symbols-outlined" style="font-size:16px">"call"</span>
                                                    {log.phone.clone()}
                                                </div>
                                                <p class="text-label-sm text-surface-on-variant/60">{date.to_string()}</p>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
