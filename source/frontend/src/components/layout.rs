use leptos::prelude::*;

use super::navbar::Navbar;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-surface flex flex-col">
            <Navbar />
            <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 py-4 sm:py-8">
                {children()}
            </main>
        </div>
    }
}
