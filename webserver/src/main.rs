use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use wasmi::{Caller, Engine, Func, Linker, Module, Store};

// Shared state for the Wasmi caller context mapping your BPM state
struct WorkflowContext {
    process_id: String,
    completed_steps: Vec<i32>,
}

// Axum Request Payload
#[derive(Deserialize)]
struct TriggerWorkflowRequest {
    process_id: String,
    completed_steps: Vec<i32>,
}

// Axum Response Payload
#[derive(Serialize)]
struct WorkflowResponse {
    status: String,
    message: String,
}

#[tokio::main]
async fn main() {
    // 1. Build the Axum Router with standard tracing layers
    let app = Router::new()
        .route("/api/workflow/run", post(handle_run_workflow))
        .layer(TraceLayer::new_for_http());

    // 2. Start the high-performance Tokio TCP listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("[Webserver] Async production engine online at http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}

// The Async Axum Handler executing your Wasmi runtime inside a worker thread block
async fn handle_run_workflow(
    Json(payload): Json<TriggerWorkflowRequest>,
) -> Json<WorkflowResponse> {
    // Read the compiled WASM binary inside our async loop
    let wasm_bytes = match std::fs::read("target/wasm32-unknown-unknown/release/examples_runner.wasm") {
        Ok(bytes) => bytes,
        Err(_) => {
            return Json(WorkflowResponse {
                status: "error".to_string(),
                message: "Wasm binary target file missing. Please build examples-runner.".to_string(),
            });
        }
    };

    // Execute Wasmi logic inside a blocking task thread to keep the main async event loop unblocked
    let result = tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // Inject the incoming JSON request state context directly into the Wasm Store runtime
        let state = WorkflowContext {
            process_id: payload.process_id,
            completed_steps: payload.completed_steps,
        };
        let mut store = Store::new(&engine, state);

        // Bind 'host_log_step' targeting the module configuration in plugin-sdk
        linker.define(
            "bpm_host",
            "host_log_step",
            Func::wrap(&mut store, |caller: Caller<WorkflowContext>, step_id: i32| {
                let ctx = caller.data();
                println!("[BPM Event Log] Process '{}' updated step milestone: {}", ctx.process_id, step_id);
            }),
        ).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Bind 'host_get_step_status' targeting the module configuration in plugin-sdk
        linker.define(
            "bpm_host",
            "host_get_step_status",
            Func::wrap(&mut store, |caller: Caller<WorkflowContext>, step_id: i32| -> i32 {
                let ctx = caller.data();
                if ctx.completed_steps.contains(&step_id) { 1 } else { 0 }
            }),
        ).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Compile and instantiate the guest workflow
        let module = Module::new(&engine, &wasm_bytes[..]).map_err(|e| Box::new(e))?;
        let instance = linker.instantiate(&mut store, &module).map_err(|e| Box::new(e))?.start(&mut store).map_err(|e| Box::new(e))?;

        let execute_workflow = instance
            .get_export(&store, "execute_bpm_workflow")
            .and_then(|ext| ext.into_func())
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Export execute_bpm_workflow missing")) as Box<dyn std::error::Error + Send + Sync>)?;

        // Execute the sandbox runtime
        execute_workflow.call(&mut store, &[], &mut []).map_err(|e| Box::new(e))?;
        Ok(())
    }).await;

    match result {
        Ok(Ok(())) => Json(WorkflowResponse {
            status: "success".to_string(),
            message: "Wasm workflow pipeline evaluated successfully".to_string(),
        }),
        _ => Json(WorkflowResponse {
            status: "error".to_string(),
            message: "Failed during isolation execution evaluation engine phase".to_string(),
        }),
    }
}
