mod components;
mod pages;
mod services;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::*;
use leptos_router::path;

use pages::login::LoginPage;
use pages::home::HomePage;
use pages::users::UsersPage;
use pages::products::ProductsPage;
use pages::musteri_kabul::MusteriKabulPage;
use pages::montaj::MontajPage;
use pages::irsaliye::IrsaliyePage;
use pages::settings::SettingsPage;
use pages::servis_kayitlari::ServisKayitlariPage;
use components::layout::Layout;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

/// localStorage'daki auth token'ı oku
fn get_stored_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("auth_token").ok()?
}

#[component]
fn App() -> impl IntoView {
    // Auth durumu: None = kontrol ediliyor, Some(false) = giriş yok, Some(true) = giriş yapıldı
    let (auth_state, set_auth_state) = signal(Option::<bool>::None);

    // Sayfa yüklenince token'ı kontrol et
    spawn_local(async move {
        if let Some(token) = get_stored_token() {
            match services::api::check_auth(&token).await {
                Ok(_) => set_auth_state.set(Some(true)),
                Err(_) => {
                    // Geçersiz token — temizle
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.remove_item("auth_token");
                            let _ = storage.remove_item("auth_user");
                        }
                    }
                    set_auth_state.set(Some(false));
                }
            }
        } else {
            set_auth_state.set(Some(false));
        }
    });

    view! {
        {move || match auth_state.get() {
            None => {
                // Yükleniyor
                view! {
                    <div class="min-h-screen bg-surface flex items-center justify-center">
                        <span class="material-symbols-outlined animate-spin text-primary" style="font-size:40px">"progress_activity"</span>
                    </div>
                }.into_any()
            }
            Some(false) => {
                // Giriş yapılmamış
                view! { <LoginPage /> }.into_any()
            }
            Some(true) => {
                // Giriş yapılmış — normal router
                view! {
                    <Router>
                        <Layout>
                            <Routes fallback=|| view! { <div class="flex-1 flex items-center justify-center"><p class="text-headline-sm text-surface-on-variant">"404 — Sayfa bulunamadı"</p></div> }>
                                <Route path=path!("/") view=HomePage />
                                <Route path=path!("/musteri-kabul") view=MusteriKabulPage />
                                <Route path=path!("/montaj") view=MontajPage />
                                <Route path=path!("/irsaliye") view=IrsaliyePage />
                                <Route path=path!("/servis-kayitlari/:status") view=ServisKayitlariPage />
                                <Route path=path!("/settings") view=SettingsPage />
                                <Route path=path!("/users") view=UsersPage />
                                <Route path=path!("/products") view=ProductsPage />
                            </Routes>
                        </Layout>
                    </Router>
                }.into_any()
            }
        }}
    }
}
