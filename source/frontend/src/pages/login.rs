use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::services::api;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_loading.set(true);

        let u = username.get_untracked();
        let p = password.get_untracked();

        spawn_local(async move {
            match api::login(&u, &p).await {
                Ok(resp) => {
                    // Token'ı localStorage'a kaydet
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.set_item("auth_token", &resp.token);
                            let _ = storage.set_item("auth_user", &resp.username);
                        }
                    }
                    // Sayfayı yenile → App auth kontrolü devreye girer
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-surface flex items-center justify-center px-4">
            <div class="w-full max-w-sm">
                // Logo / Brand
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary-container mb-4">
                        <span class="material-symbols-outlined text-primary-on-container" style="font-size:32px">"bolt"</span>
                    </div>
                    <h1 class="text-headline-md text-surface-on">"SIS Teknik"</h1>
                    <p class="text-body-md text-surface-on-variant mt-1">"Servis Yönetim Sistemi"</p>
                </div>

                // Login Card
                <div class="surface-1 rounded-xl shadow-elevation-2 p-6">
                    <h2 class="text-title-lg text-surface-on mb-6">"Giriş Yap"</h2>

                    <form on:submit=on_submit class="space-y-4">
                        // Username
                        <div>
                            <label class="block text-label-md text-surface-on-variant mb-1.5">"Kullanıcı Adı"</label>
                            <div class="relative">
                                <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-surface-on-variant" style="font-size:20px">"person"</span>
                                <input
                                    type="text"
                                    required
                                    autocomplete="username"
                                    prop:value=move || username.get()
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    class="w-full pl-10 pr-4 h-12 rounded-lg border border-outline bg-surface text-surface-on text-body-lg placeholder:text-surface-on-variant/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
                                    placeholder="admin"
                                />
                            </div>
                        </div>

                        // Password
                        <div>
                            <label class="block text-label-md text-surface-on-variant mb-1.5">"Şifre"</label>
                            <div class="relative">
                                <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-surface-on-variant" style="font-size:20px">"lock"</span>
                                <input
                                    type="password"
                                    required
                                    autocomplete="current-password"
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    class="w-full pl-10 pr-4 h-12 rounded-lg border border-outline bg-surface text-surface-on text-body-lg placeholder:text-surface-on-variant/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
                                    placeholder="\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}"
                                />
                            </div>
                        </div>

                        // Error message
                        {move || error.get().map(|e| view! {
                            <div class="flex items-center gap-2 p-3 rounded-lg bg-error-container text-error-on-container">
                                <span class="material-symbols-outlined" style="font-size:18px">"error"</span>
                                <span class="text-body-sm">{e}</span>
                            </div>
                        })}

                        // Submit button
                        <button
                            type="submit"
                            disabled=move || loading.get()
                            class="w-full h-12 rounded-full bg-primary text-primary-on text-label-lg font-medium hover:shadow-elevation-1 active:shadow-elevation-0 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                        >
                            {move || if loading.get() {
                                view! {
                                    <span class="material-symbols-outlined animate-spin" style="font-size:20px">"progress_activity"</span>
                                    <span>"Giriş yapılıyor..."</span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="material-symbols-outlined" style="font-size:20px">"login"</span>
                                    <span>"Giriş Yap"</span>
                                }.into_any()
                            }}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
