use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_location;

fn do_logout() {
    spawn_local(async move {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let token = storage.get_item("auth_token").ok().flatten().unwrap_or_default();
                let _ = crate::services::api::logout_api(&token).await;
                let _ = storage.remove_item("auth_token");
                let _ = storage.remove_item("auth_user");
            }
            let _ = window.location().set_href("/");
        }
    });
}

#[component]
pub fn Navbar() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);
    let location = use_location();

    // Close mobile menu on nav click
    let close_menu = move |_| set_mobile_open.set(false);

    view! {
        // MD3 Top App Bar
        <header class="surface-2 shadow-elevation-2 sticky top-0 z-50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6">
                <div class="flex items-center h-16 gap-2">

                    // Mobile hamburger button
                    <button
                        on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                        class="state-layer md:hidden flex items-center justify-center w-10 h-10 rounded-full text-surface-on"
                        aria-label="Men\u{00fc}"
                    >
                        <span class="material-symbols-outlined" style="font-size:24px">
                            {move || if mobile_open.get() { "close" } else { "menu" }}
                        </span>
                    </button>

                    // Brand
                    <a href="/" class="state-layer flex items-center justify-center w-10 h-10 sm:w-12 sm:h-12 rounded-full text-surface-on">
                        <span class="material-symbols-outlined" style="font-size:24px">"bolt"</span>
                    </a>

                    // Title
                    <span class="text-title-md sm:text-title-lg text-surface-on select-none">
                        "SIS Teknik"
                    </span>

                    // Desktop nav (centered)
                    <nav class="hidden md:flex flex-1 items-center justify-center gap-1 h-full">
                        <NavItem href="/" label="Anasayfa" icon="home" location=location.clone() />
                        <NavItem href="/servis-kayitlari/musteri_kabul?yeni=true" label="M\u{00fc}\u{015f}teri Kabul" icon="person_add" location=location.clone() />
                        <NavItem href="/montaj" label="Montaj" icon="construction" location=location.clone() />
                        <NavItem href="/irsaliye" label="\u{0130}rsaliye" icon="local_shipping" location=location.clone() />
                    </nav>

                    // Trailing icons — settings & logout
                    <div class="hidden md:flex items-center gap-1 ml-auto">
                        <a
                            href="/settings"
                            class=move || {
                                let path = (location.pathname).get();
                                if path == "/settings" {
                                    "state-layer flex items-center justify-center w-10 h-10 rounded-full text-primary transition-colors"
                                } else {
                                    "state-layer flex items-center justify-center w-10 h-10 rounded-full text-surface-on-variant hover:text-surface-on transition-colors"
                                }
                            }
                        >
                            <span class="material-symbols-outlined" style="font-size:22px">"settings"</span>
                        </a>
                        <button
                            on:click=move |_| do_logout()
                            class="state-layer flex items-center justify-center w-10 h-10 rounded-full text-surface-on-variant hover:text-error transition-colors">
                            <span class="material-symbols-outlined" style="font-size:22px">"logout"</span>
                        </button>
                    </div>

                </div>
            </div>

            // Mobile nav drawer
            <div class=move || {
                if mobile_open.get() {
                    "md:hidden border-t border-outline-variant surface-1"
                } else {
                    "hidden"
                }
            }>
                <nav class="px-4 py-3 space-y-1">
                    <MobileNavItem href="/" label="Anasayfa" icon="home" on_click=close_menu location=location.clone() />
                    <MobileNavItem href="/servis-kayitlari/musteri_kabul?yeni=true" label="M\u{00fc}\u{015f}teri Kabul" icon="person_add" on_click=close_menu location=location.clone() />
                    <MobileNavItem href="/montaj" label="Montaj" icon="construction" on_click=close_menu location=location.clone() />
                    <MobileNavItem href="/irsaliye" label="\u{0130}rsaliye" icon="local_shipping" on_click=close_menu location=location.clone() />
                    <div class="border-t border-outline-variant my-2"></div>
                    <MobileNavItem href="/settings" label="Ayarlar" icon="settings" on_click=close_menu location=location.clone() />
                    <a
                        href="#"
                        on:click=move |ev: web_sys::MouseEvent| { ev.prevent_default(); do_logout(); }
                        class="state-layer flex items-center gap-3 px-4 h-12 rounded-md text-error hover:text-error transition-colors duration-200 w-full"
                    >
                        <span class="material-symbols-outlined" style="font-size:22px">"logout"</span>
                        <span class="text-body-lg">"\u{00c7}\u{0131}k\u{0131}\u{015f}"</span>
                    </a>
                </nav>
            </div>
        </header>
    }
}

#[component]
fn NavItem(href: &'static str, label: &'static str, icon: &'static str, location: leptos_router::location::Location) -> impl IntoView {
    let is_active = move || {
        let path = (location.pathname).get();
        if href == "/" { path == "/" } else { path.starts_with(href) }
    };

    view! {
        <a
            href=href
            class=move || {
                if is_active() {
                    "state-layer group flex items-center gap-2 px-4 h-10 rounded-full bg-primary-container text-primary-on-container transition-colors duration-200"
                } else {
                    "state-layer group flex items-center gap-2 px-4 h-10 rounded-full text-surface-on-variant hover:text-surface-on transition-colors duration-200"
                }
            }
        >
            <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
            <span class="text-label-lg">{label}</span>
        </a>
    }
}

#[component]
fn MobileNavItem(
    href: &'static str,
    label: &'static str,
    icon: &'static str,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
    location: leptos_router::location::Location,
) -> impl IntoView {
    let is_active = move || {
        let path = (location.pathname).get();
        if href == "/" { path == "/" } else { path.starts_with(href) }
    };

    view! {
        <a
            href=href
            on:click=on_click
            class=move || {
                if is_active() {
                    "state-layer flex items-center gap-3 px-4 h-12 rounded-md bg-primary-container text-primary-on-container transition-colors duration-200 w-full"
                } else {
                    "state-layer flex items-center gap-3 px-4 h-12 rounded-md text-surface-on-variant hover:text-surface-on transition-colors duration-200 w-full"
                }
            }
        >
            <span class="material-symbols-outlined" style="font-size:22px">{icon}</span>
            <span class="text-body-lg">{label}</span>
        </a>
    }
}
