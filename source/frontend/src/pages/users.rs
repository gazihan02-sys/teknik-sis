use leptos::prelude::*;
use crate::services::api;
use shared::dto::user_dto::{CreateUserRequest, UserResponse};

#[component]
pub fn UsersPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<UserResponse>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_form, set_show_form) = signal(false);

    // Form fields
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (role, set_role) = signal("user".to_string());

    // Load users
    let load_users = move || {
        set_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match api::get_users().await {
                Ok(data) => {
                    set_users.set(data);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    // Initial load
    load_users();

    // Create user
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let req = CreateUserRequest {
            username: username.get_untracked(),
            email: email.get_untracked(),
            role: Some(role.get_untracked()),
        };
        leptos::task::spawn_local(async move {
            match api::create_user(req).await {
                Ok(user) => {
                    set_users.update(|u| u.insert(0, user));
                    set_username.set(String::new());
                    set_email.set(String::new());
                    set_role.set("user".to_string());
                    set_show_form.set(false);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    // Delete user
    let delete_user = move |id: String| {
        leptos::task::spawn_local(async move {
            if api::delete_user(&id).await.is_ok() {
                set_users.update(|u| u.retain(|user| user.id != id));
            }
        });
    };

    view! {
        <div class="space-y-4 sm:space-y-6">
            // Header
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                <h1 class="text-headline-sm sm:text-headline-md text-surface-on">"Kullan\u{0131}c\u{0131}lar"</h1>
                <button
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                    class="state-layer inline-flex items-center justify-center gap-2 bg-primary text-primary-on px-5 sm:px-6 py-2.5 rounded-full text-label-lg transition-colors w-full sm:w-auto"
                >
                    <span class="material-symbols-outlined" style="font-size:18px">
                        {move || if show_form.get() { "close" } else { "add" }}
                    </span>
                    {move || if show_form.get() { "\u{0130}ptal" } else { "Yeni Kullan\u{0131}c\u{0131}" }}
                </button>
            </div>

            // Error display
            {move || error.get().map(|e| view! {
                <div class="bg-error-container text-error-on-container px-4 py-3 rounded-md text-body-md">
                    {e}
                </div>
            })}

            // Create form
            {move || show_form.get().then(|| view! {
                <form on:submit=on_submit class="surface-1 rounded-lg sm:rounded-xl shadow-elevation-1 p-4 sm:p-6 space-y-4">
                    <h2 class="text-title-lg text-surface-on">"Yeni Kullan\u{0131}c\u{0131}"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"Kullan\u{0131}c\u{0131} Ad\u{0131}"</label>
                            <input
                                type="text"
                                prop:value=move || username.get()
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                class="md3-input"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"Email"</label>
                            <input
                                type="email"
                                prop:value=move || email.get()
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                class="md3-input"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"Rol"</label>
                            <select
                                prop:value=move || role.get()
                                on:change=move |ev| set_role.set(event_target_value(&ev))
                                class="md3-input"
                            >
                                <option value="user">"Kullan\u{0131}c\u{0131}"</option>
                                <option value="admin">"Admin"</option>
                                <option value="viewer">"\u{0130}zleyici"</option>
                            </select>
                        </div>
                    </div>
                    <button
                        type="submit"
                        class="state-layer inline-flex items-center gap-2 bg-primary text-primary-on px-6 py-2.5 rounded-full text-label-lg w-full sm:w-auto justify-center"
                    >
                        <span class="material-symbols-outlined" style="font-size:18px">"save"</span>
                        "Kaydet"
                    </button>
                </form>
            })}

            // Loading
            {move || loading.get().then(|| view! {
                <div class="text-center py-8">
                    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                    <p class="text-body-md text-surface-on-variant mt-2">"Y\u{00fc}kleniyor..."</p>
                </div>
            })}

            // Users — Desktop table / Mobile cards
            {move || (!loading.get()).then(|| view! {
                // Desktop table (hidden on mobile)
                <div class="hidden md:block surface-1 rounded-xl shadow-elevation-1 overflow-hidden">
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[600px]">
                            <thead class="bg-surface-container-high">
                                <tr>
                                    <th class="px-4 lg:px-6 py-3 text-left text-label-lg text-surface-on-variant">"Kullan\u{0131}c\u{0131} Ad\u{0131}"</th>
                                    <th class="px-4 lg:px-6 py-3 text-left text-label-lg text-surface-on-variant">"Email"</th>
                                    <th class="px-4 lg:px-6 py-3 text-left text-label-lg text-surface-on-variant">"Rol"</th>
                                    <th class="px-4 lg:px-6 py-3 text-left text-label-lg text-surface-on-variant">"Olu\u{015f}turulma"</th>
                                    <th class="px-4 lg:px-6 py-3 text-right text-label-lg text-surface-on-variant">"\u{0130}\u{015f}lemler"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant">
                                {move || users.get().into_iter().map(|user| {
                                    let id_for_delete = user.id.clone();
                                    view! {
                                        <tr class="hover:bg-surface-container-low transition-colors">
                                            <td class="px-4 lg:px-6 py-4 text-body-md text-surface-on font-medium">{user.username}</td>
                                            <td class="px-4 lg:px-6 py-4 text-body-md text-surface-on-variant">{user.email}</td>
                                            <td class="px-4 lg:px-6 py-4">
                                                <span class="px-3 py-1 text-label-md rounded-full bg-tertiary-container text-tertiary-on-container">
                                                    {user.role}
                                                </span>
                                            </td>
                                            <td class="px-4 lg:px-6 py-4 text-body-sm text-surface-on-variant">{user.created_at}</td>
                                            <td class="px-4 lg:px-6 py-4 text-right">
                                                <button
                                                    on:click=move |_| delete_user(id_for_delete.clone())
                                                    class="state-layer inline-flex items-center gap-1 text-error hover:text-error text-label-lg px-3 py-1.5 rounded-full"
                                                >
                                                    <span class="material-symbols-outlined" style="font-size:18px">"delete"</span>
                                                    "Sil"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>

                // Mobile cards (hidden on desktop)
                <div class="md:hidden space-y-3">
                    {move || users.get().into_iter().map(|user| {
                        let id_for_delete = user.id.clone();
                        let username = user.username.clone();
                        let email = user.email.clone();
                        let role = user.role.clone();
                        let created_at = user.created_at.clone();
                        view! {
                            <div class="surface-1 rounded-lg shadow-elevation-1 p-4">
                                <div class="flex items-start justify-between gap-3 mb-2">
                                    <div class="min-w-0 flex-1">
                                        <p class="text-title-md text-surface-on truncate">{username}</p>
                                        <p class="text-body-sm text-surface-on-variant truncate">{email}</p>
                                    </div>
                                    <span class="shrink-0 px-3 py-1 text-label-sm rounded-full bg-tertiary-container text-tertiary-on-container">
                                        {role}
                                    </span>
                                </div>
                                <div class="flex items-center justify-between mt-3 pt-3 border-t border-outline-variant">
                                    <span class="text-body-sm text-surface-on-variant">{created_at}</span>
                                    <button
                                        on:click=move |_| delete_user(id_for_delete.clone())
                                        class="state-layer inline-flex items-center gap-1 text-error text-label-lg px-3 py-1.5 rounded-full"
                                    >
                                        <span class="material-symbols-outlined" style="font-size:18px">"delete"</span>
                                        "Sil"
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                {move || users.get().is_empty().then(|| view! {
                    <div class="text-center py-8 text-body-lg text-surface-on-variant">"Hen\u{00fc}z kullan\u{0131}c\u{0131} bulunmuyor."</div>
                })}
            })}
        </div>
    }
}
