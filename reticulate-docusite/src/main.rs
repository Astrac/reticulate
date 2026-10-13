mod app;
mod markdown;
mod page;
mod toc;

use dioxus::prelude::*;

fn main() {
    #[cfg(all(not(target_arch = "wasm32"), not(debug_assertions)))]
    {
        dioxus::LaunchBuilder::new()
            .with_cfg(server_only! {
                ServeConfig::builder()
                    .incremental(
                        dioxus::server::IncrementalRendererConfig::new()
                            .static_dir(
                                std::env::current_exe()
                                    .unwrap()
                                    .parent()
                                    .unwrap()
                                    .join("public")
                            )
                            .clear_cache(false)
                    )
                    .enable_out_of_order_streaming()
            })
            .launch(crate::app::App);
    }

    #[cfg(any(target_arch = "wasm32", debug_assertions))]
    {
        dioxus::launch(crate::app::App);
    }
}

#[server(endpoint = "static_routes", output = server_fn::codec::Json)]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    Ok(crate::app::Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect())
}
