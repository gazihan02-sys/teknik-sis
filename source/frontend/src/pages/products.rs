use leptos::prelude::*;
use crate::services::api;
use shared::dto::product_dto::{CreateProductRequest, ProductResponse};

#[component]
pub fn ProductsPage() -> impl IntoView {
    let (products, set_products) = signal(Vec::<ProductResponse>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_form, set_show_form) = signal(false);

    // Form fields
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (price, set_price) = signal(String::new());
    let (stock, set_stock) = signal(String::new());
    let (category, set_category) = signal(String::new());

    // Load products
    let load_products = move || {
        set_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match api::get_products().await {
                Ok(data) => {
                    set_products.set(data);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    load_products();

    // Create product
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let req = CreateProductRequest {
            name: name.get_untracked(),
            description: description.get_untracked(),
            price: price.get_untracked().parse().unwrap_or(0.0),
            stock: stock.get_untracked().parse().unwrap_or(0),
            category: category.get_untracked(),
        };
        leptos::task::spawn_local(async move {
            match api::create_product(req).await {
                Ok(product) => {
                    set_products.update(|p| p.insert(0, product));
                    set_name.set(String::new());
                    set_description.set(String::new());
                    set_price.set(String::new());
                    set_stock.set(String::new());
                    set_category.set(String::new());
                    set_show_form.set(false);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let delete_product = move |id: String| {
        leptos::task::spawn_local(async move {
            if api::delete_product(&id).await.is_ok() {
                set_products.update(|p| p.retain(|prod| prod.id != id));
            }
        });
    };

    view! {
        <div class="space-y-4 sm:space-y-6">
            // Header
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                <h1 class="text-headline-sm sm:text-headline-md text-surface-on">"\u{00dc}r\u{00fc}nler"</h1>
                <button
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                    class="state-layer inline-flex items-center justify-center gap-2 bg-primary text-primary-on px-5 sm:px-6 py-2.5 rounded-full text-label-lg transition-colors w-full sm:w-auto"
                >
                    <span class="material-symbols-outlined" style="font-size:18px">
                        {move || if show_form.get() { "close" } else { "add" }}
                    </span>
                    {move || if show_form.get() { "\u{0130}ptal" } else { "Yeni \u{00dc}r\u{00fc}n" }}
                </button>
            </div>

            // Error
            {move || error.get().map(|e| view! {
                <div class="bg-error-container text-error-on-container px-4 py-3 rounded-md text-body-md">{e}</div>
            })}

            // Form
            {move || show_form.get().then(|| view! {
                <form on:submit=on_submit class="surface-1 rounded-lg sm:rounded-xl shadow-elevation-1 p-4 sm:p-6 space-y-4">
                    <h2 class="text-title-lg text-surface-on">"Yeni \u{00dc}r\u{00fc}n"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"\u{00dc}r\u{00fc}n Ad\u{0131}"</label>
                            <input
                                type="text"
                                prop:value=move || name.get()
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                class="md3-input"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"Kategori"</label>
                            <input
                                type="text"
                                prop:value=move || category.get()
                                on:input=move |ev| set_category.set(event_target_value(&ev))
                                class="md3-input"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"Fiyat (\u{20ba})"</label>
                            <input
                                type="number"
                                step="0.01"
                                prop:value=move || price.get()
                                on:input=move |ev| set_price.set(event_target_value(&ev))
                                class="md3-input"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-label-lg text-surface-on-variant mb-1.5">"Stok"</label>
                            <input
                                type="number"
                                prop:value=move || stock.get()
                                on:input=move |ev| set_stock.set(event_target_value(&ev))
                                class="md3-input"
                                required
                            />
                        </div>
                    </div>
                    <div>
                        <label class="block text-label-lg text-surface-on-variant mb-1.5">"A\u{00e7}\u{0131}klama"</label>
                        <textarea
                            prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            class="md3-input"
                            rows="3"
                        ></textarea>
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

            // Product cards
            {move || (!loading.get()).then(|| view! {
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
                    {move || products.get().into_iter().map(|product| {
                        let id_for_delete = product.id.clone();
                        let name = product.name.clone();
                        let category = product.category.clone();
                        let description = product.description.clone();
                        let price_str = format!("\u{20ba}{:.2}", product.price);
                        let stock_str = format!("Stok: {}", product.stock);
                        view! {
                            <div class="surface-1 rounded-lg sm:rounded-xl shadow-elevation-1 hover:shadow-elevation-2 p-4 sm:p-6 transition-shadow">
                                <div class="flex items-start justify-between gap-2 mb-3">
                                    <h3 class="text-title-md text-surface-on min-w-0 truncate">{name}</h3>
                                    <span class="shrink-0 px-3 py-1 text-label-sm rounded-full bg-secondary-container text-secondary-on-container">
                                        {category}
                                    </span>
                                </div>
                                <p class="text-body-md text-surface-on-variant mb-4 line-clamp-2">{description}</p>
                                <div class="flex items-end justify-between gap-3">
                                    <div>
                                        <p class="text-headline-sm text-primary">{price_str}</p>
                                        <p class="text-body-sm text-surface-on-variant">{stock_str}</p>
                                    </div>
                                    <button
                                        on:click=move |_| delete_product(id_for_delete.clone())
                                        class="state-layer inline-flex items-center gap-1 text-error text-label-lg px-3 py-1.5 rounded-full shrink-0"
                                    >
                                        <span class="material-symbols-outlined" style="font-size:18px">"delete"</span>
                                        "Sil"
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                {move || products.get().is_empty().then(|| view! {
                    <div class="text-center py-8 text-body-lg text-surface-on-variant">"Hen\u{00fc}z \u{00fc}r\u{00fc}n bulunmuyor."</div>
                })}
            })}
        </div>
    }
}
