use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = localStorage, js_name = getItem)]
    fn local_storage_get(key: &str) -> Option<String>;
    #[wasm_bindgen(js_namespace = localStorage, js_name = setItem)]
    fn local_storage_set(key: &str, value: &str);
}

fn get_current_theme() -> String {
    local_storage_get("md3-theme").unwrap_or_else(|| "purple".to_string())
}

fn apply_theme(theme: &str) {
    local_storage_set("md3-theme", theme);
    let document = web_sys::window().unwrap().document().unwrap();
    let html = document.document_element().unwrap();
    if theme == "purple" {
        html.remove_attribute("data-theme").ok();
    } else {
        html.set_attribute("data-theme", theme).ok();
    }
}

fn theme_label(id: &str) -> &'static str {
    match id {
        "purple" => "Mor",
        "blue" => "Mavi",
        "indigo" => "\u{0130}ndigo",
        "cyan" => "Camg\u{00f6}be\u{011f}i",
        "teal" => "Deniz Ye\u{015f}ili",
        "green" => "Ye\u{015f}il",
        "orange" => "Turuncu",
        "red" => "K\u{0131}rm\u{0131}z\u{0131}",
        "rose" => "G\u{00fc}l",
        "pink" => "Pembe",
        "fuchsia" => "Fuşya",
        "amber" => "Kehribar",
        _ => "",
    }
}

struct ThemeInfo {
    id: &'static str,
    label: &'static str,
    color: &'static str,
}

const THEMES: &[ThemeInfo] = &[
    ThemeInfo { id: "purple", label: "Mor", color: "#6750A4" },
    ThemeInfo { id: "blue", label: "Mavi", color: "#0061A4" },
    ThemeInfo { id: "indigo", label: "\u{0130}ndigo", color: "#415AA9" },
    ThemeInfo { id: "cyan", label: "Camg\u{00f6}be\u{011f}i", color: "#006879" },
    ThemeInfo { id: "teal", label: "Deniz Ye\u{015f}ili", color: "#006A6A" },
    ThemeInfo { id: "green", label: "Ye\u{015f}il", color: "#386A20" },
    ThemeInfo { id: "orange", label: "Turuncu", color: "#8B5000" },
    ThemeInfo { id: "red", label: "K\u{0131}rm\u{0131}z\u{0131}", color: "#BE0E13" },
    ThemeInfo { id: "rose", label: "G\u{00fc}l", color: "#984061" },
    ThemeInfo { id: "pink", label: "Pembe", color: "#BC004B" },
    ThemeInfo { id: "fuchsia", label: "Fu\u{015f}ya", color: "#9A25AE" },
    ThemeInfo { id: "amber", label: "Kehribar", color: "#785900" },
];

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (active_theme, set_active_theme) = signal(get_current_theme());
    let (modal_open, set_modal_open) = signal(false);

    let on_select = move |theme_id: &'static str| {
        apply_theme(theme_id);
        set_active_theme.set(theme_id.to_string());
    };

    view! {
        <div class="max-w-3xl mx-auto">
            <h1 class="text-headline-sm sm:text-headline-md text-surface-on mb-6">"Ayarlar"</h1>

            // Settings cards grid
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">

                // Renk Teması card
                <button
                    on:click=move |_| set_modal_open.set(true)
                    class="state-layer bg-surface-container-low rounded-lg p-4 text-left shadow-elevation-1 hover:shadow-elevation-2 transition-all duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                >
                    <div class="flex items-start gap-3">
                        <div class="w-10 h-10 rounded-full bg-primary flex items-center justify-center shrink-0">
                            <span class="material-symbols-outlined text-primary-on" style="font-size:20px">"palette"</span>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h3 class="text-title-md text-surface-on">"Renk Temas\u{0131}"</h3>
                            <p class="text-body-sm text-surface-on-variant mt-0.5">
                                {move || {
                                    let current = active_theme.get();
                                    let label = theme_label(&current);
                                    format!("Aktif: {label}")
                                }}
                            </p>
                        </div>
                        <span class="material-symbols-outlined text-surface-on-variant shrink-0" style="font-size:20px">"chevron_right"</span>
                    </div>
                </button>

                // Dil card (placeholder)
                <div class="bg-surface-container-low rounded-lg p-4 shadow-elevation-1 opacity-60">
                    <div class="flex items-start gap-3">
                        <div class="w-10 h-10 rounded-full bg-tertiary flex items-center justify-center shrink-0">
                            <span class="material-symbols-outlined text-tertiary-on" style="font-size:20px">"translate"</span>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h3 class="text-title-md text-surface-on">"Dil"</h3>
                            <p class="text-body-sm text-surface-on-variant mt-0.5">"T\u{00fc}rk\u{00e7}e"</p>
                        </div>
                    </div>
                </div>

                // Hesap card (placeholder)
                <div class="bg-surface-container-low rounded-lg p-4 shadow-elevation-1 opacity-60">
                    <div class="flex items-start gap-3">
                        <div class="w-10 h-10 rounded-full bg-surface-container-highest flex items-center justify-center shrink-0">
                            <span class="material-symbols-outlined text-surface-on-variant" style="font-size:20px">"person"</span>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h3 class="text-title-md text-surface-on">"Hesap"</h3>
                            <p class="text-body-sm text-surface-on-variant mt-0.5">"Yak\u{0131}nda"</p>
                        </div>
                    </div>
                </div>

            </div>
        </div>

        // ========== Renk Teması Modal ==========
        {move || {
            if modal_open.get() {
                view! {
                    // Backdrop
                    <div
                        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50"
                        on:click=move |e| {
                            // Close if clicked on backdrop itself
                            if event_target::<web_sys::HtmlElement>(&e).class_list().contains("fixed") {
                                set_modal_open.set(false);
                            }
                        }
                    >
                        // Dialog
                        <div class="bg-surface-container-high rounded-xl shadow-elevation-3 w-full max-w-md max-h-[80vh] overflow-y-auto animate-in">
                            // Header
                            <div class="flex items-center justify-between px-6 pt-6 pb-2">
                                <h2 class="text-headline-sm text-surface-on">"Renk Temas\u{0131}"</h2>
                                <button
                                    on:click=move |_| set_modal_open.set(false)
                                    class="state-layer w-10 h-10 rounded-full flex items-center justify-center text-surface-on-variant hover:text-surface-on transition-colors"
                                >
                                    <span class="material-symbols-outlined" style="font-size:22px">"close"</span>
                                </button>
                            </div>
                            <p class="px-6 pb-4 text-body-md text-surface-on-variant">
                                "Uygulaman\u{0131}n renk palettini se\u{00e7}in."
                            </p>

                            // Theme grid
                            <div class="px-6 pb-6 grid grid-cols-3 gap-3">
                                {THEMES.iter().map(|t| {
                                    let theme_id = t.id;
                                    let label = t.label;
                                    let color = t.color.to_string();
                                    let on_select = on_select.clone();
                                    view! {
                                        <button
                                            on:click=move |_| on_select(theme_id)
                                            class="flex flex-col items-center gap-2 p-3 rounded-lg transition-colors duration-200 hover:bg-surface-container-highest focus:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                                        >
                                            <div
                                                class=move || {
                                                    let base = "w-12 h-12 rounded-full transition-all duration-200 shadow-elevation-1 flex items-center justify-center";
                                                    if active_theme.get() == theme_id {
                                                        format!("{base} ring-3 ring-primary ring-offset-2 ring-offset-surface-container-high scale-110")
                                                    } else {
                                                        format!("{base} hover:scale-105")
                                                    }
                                                }
                                                style=format!("background-color: {color}")
                                            >
                                                {move || {
                                                    if active_theme.get() == theme_id {
                                                        view! {
                                                            <span class="material-symbols-outlined text-white" style="font-size:20px">"check"</span>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }
                                                }}
                                            </div>
                                            <span class=move || {
                                                if active_theme.get() == theme_id {
                                                    "text-label-md text-primary font-medium"
                                                } else {
                                                    "text-label-md text-surface-on-variant"
                                                }
                                            }>
                                                {label}
                                            </span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div class="hidden"></div> }.into_any()
            }
        }}
    }
}
