use gloo_net::http::Request;
use leptos::*;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "/api";

fn main() {
    mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let games = create_resource(|| (), |_| async { fetch_games().await });
    let definitions_refresh = create_rw_signal(0u32);
    let model_definitions = create_resource(move || definitions_refresh.get(), |_| async {
        fetch_model_definitions().await
    });
    let models_refresh = create_rw_signal(0u32);
    let models = create_rw_signal(Vec::<UserModelListItem>::new());
    let models_loading = create_rw_signal(true);
    let models_error = create_rw_signal(None::<String>);

    let model_name = create_rw_signal(String::new());
    let selected_game_id = create_rw_signal(String::new());
    let quantity = create_rw_signal(1_i32);
    let status = create_rw_signal(Status::Unassembled);
    let form_error = create_rw_signal(None::<String>);
    let submitting = create_rw_signal(false);

    let load_models = {
        let models = models.clone();
        let models_loading = models_loading.clone();
        let models_error = models_error.clone();
        move || {
            models_loading.set(true);
            models_error.set(None);
            let models = models.clone();
            let models_loading = models_loading.clone();
            let models_error = models_error.clone();
            spawn_local(async move {
                match fetch_user_models().await {
                    Ok(list) => {
                        models.set(list);
                        models_loading.set(false);
                    }
                    Err(message) => {
                        models_error.set(Some(message));
                        models_loading.set(false);
                    }
                }
            });
        }
    };

    create_effect(move |_| {
        models_refresh.get();
        load_models();
    });

    create_effect(move |_| {
        if selected_game_id.get().is_empty() {
            if let Some(Ok(list)) = games.get() {
                if let Some(game) = list.first() {
                    selected_game_id.set(game.id.clone());
                }
            }
        }
    });

    view! {
        <main class="page">
            <header class="page__header">
                <h1>"Mini Tracker"</h1>
            </header>
            <section class="panel">
                <h2>"Add Model"</h2>
                <form class="form" on:submit=move |event| {
                    event.prevent_default();
                    let name = model_name.get().trim().to_string();
                    let game_id = selected_game_id.get();
                    let quantity_value = quantity.get();
                    let status_value = status.get();
                    let model_definitions = model_definitions.get();
                    let definitions_refresh = definitions_refresh.clone();
                    let models_refresh = models_refresh.clone();
                    let form_error = form_error.clone();
                    let submitting = submitting.clone();
                    if name.is_empty() {
                        form_error.set(Some("Model name is required.".to_string()));
                        return;
                    }
                    if game_id.is_empty() {
                        form_error.set(Some("Select a game.".to_string()));
                        return;
                    }
                    if quantity_value <= 0 {
                        form_error.set(Some("Quantity must be greater than 0.".to_string()));
                        return;
                    }
                    let definitions = match model_definitions {
                        Some(Ok(list)) => list,
                        Some(Err(message)) => {
                            form_error.set(Some(message));
                            return;
                        }
                        None => {
                            form_error.set(Some("Model definitions are still loading.".to_string()));
                            return;
                        }
                    };
                    form_error.set(None);
                    submitting.set(true);
                    spawn_local(async move {
                        let existing = definitions
                            .iter()
                            .find(|definition| definition.name == name && definition.game.id == game_id)
                            .cloned();
                        let definition = match existing {
                            Some(definition) => definition,
                            None => {
                                match create_model_definition(name.clone(), game_id.clone()).await {
                                    Ok(definition) => {
                                        definitions_refresh.update(|value| *value += 1);
                                        definition
                                    }
                                    Err(message) => {
                                        form_error.set(Some(message));
                                        submitting.set(false);
                                        return;
                                    }
                                }
                            }
                        };
                        if let Err(message) = create_user_model(
                            definition.id.clone(),
                            quantity_value,
                            status_value,
                        )
                        .await
                        {
                            form_error.set(Some(message));
                            submitting.set(false);
                            return;
                        }
                        models_refresh.update(|value| *value += 1);
                        model_name.set(String::new());
                        quantity.set(1);
                        status.set(Status::Unassembled);
                        submitting.set(false);
                    });
                }>
                    <label class="field">
                        <span>"Model Name"</span>
                        <input
                            type="text"
                            prop:value=move || model_name.get()
                            on:input=move |event| {
                                model_name.set(event_target_value(&event));
                            }
                        />
                    </label>
                    <label class="field">
                        <span>"Game"</span>
                        {move || match games.get() {
                            None => view! { <p class="state">"Loading games..."</p> }.into_view(),
                            Some(Err(message)) => view! { <p class="state state--error">{message}</p> }.into_view(),
                            Some(Ok(list)) => view! {
                                <select
                                    prop:value=move || selected_game_id.get()
                                    on:change=move |event| {
                                        selected_game_id.set(event_target_value(&event));
                                    }
                                >
                                    {list.into_iter().map(|game| view! {
                                        <option value={game.id.clone()}>{game.name}</option>
                                    }).collect_view()}
                                </select>
                            }.into_view(),
                        }}
                    </label>
                    <label class="field">
                        <span>"Quantity"</span>
                        <input
                            type="number"
                            min="1"
                            prop:value=move || quantity.get().to_string()
                            on:input=move |event| {
                                let value = event_target_value(&event).parse::<i32>().unwrap_or(1);
                                quantity.set(value);
                            }
                        />
                    </label>
                    <label class="field">
                        <span>"Status"</span>
                        <select
                            prop:value=move || status.get().as_str().to_string()
                            on:change=move |event| {
                                if let Ok(next) = event_target_value(&event).parse::<Status>() {
                                    status.set(next);
                                }
                            }
                        >
                            {Status::all().into_iter().map(|option| view! {
                                <option value={option.as_str()}>{option.label()}</option>
                            }).collect_view()}
                        </select>
                    </label>
                    <button type="submit" disabled=move || submitting.get()>
                        {move || if submitting.get() { "Saving..." } else { "Add Model" }}
                    </button>
                    {move || form_error.get().map(|message| view! {
                        <p class="state state--error">{message}</p>
                    })}
                </form>
            </section>
            <section class="panel">
                <h2>"Models"</h2>
                {move || {
                    if models_loading.get() {
                        view! { <p class="state">"Loading models..."</p> }.into_view()
                    } else if let Some(message) = models_error.get() {
                        view! { <p class="state state--error">{message}</p> }.into_view()
                    } else {
                        let on_status_change = {
                            let models = models.clone();
                            let models_error = models_error.clone();
                            Callback::new(move |(id, next_status): (String, Status)| {
                                let previous = models.get();
                                let mut updated = previous.clone();
                                if let Some(target) = updated.iter_mut().find(|item| item.id == id) {
                                    target.status = next_status;
                                    models.set(updated);
                                    let models = models.clone();
                                    let models_error = models_error.clone();
                                    spawn_local(async move {
                                        if let Err(message) =
                                            update_user_model_status(id.clone(), next_status).await
                                        {
                                            models.set(previous);
                                            models_error.set(Some(message));
                                        }
                                    });
                                }
                            })
                        };
                        view! { <ModelsTable models=models.read_only() on_status_change /> }.into_view()
                    }
                }}
            </section>
            <section class="panel panel--compact">
                <h2>"Games"</h2>
                {move || match games.get() {
                    None => view! { <p class="state">"Loading games..."</p> }.into_view(),
                    Some(Err(message)) => view! { <p class="state state--error">{message}</p> }.into_view(),
                    Some(Ok(list)) => view! {
                        <ul class="games">
                            {list.into_iter().map(|game| view! {
                                <li>{game.name}</li>
                            }).collect_view()}
                        </ul>
                    }.into_view(),
                }}
            </section>
        </main>
    }
}

#[component]
fn ModelsTable(
    models: ReadSignal<Vec<UserModelListItem>>,
    on_status_change: Callback<(String, Status)>,
) -> impl IntoView {
    view! {
        <table class="table">
            <thead>
                <tr>
                    <th>"Model Name"</th>
                    <th>"Game"</th>
                    <th class="cell-right">"Quantity"</th>
                    <th>"Status"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    models
                        .get()
                        .into_iter()
                        .map(|model| {
                            let id = model.id.clone();
                            view! {
                                <tr>
                                    <td>{model.model_name}</td>
                                    <td>{model.game_name}</td>
                                    <td class="cell-right">{model.quantity}</td>
                                    <td>
                                        <select
                                            class="inline-select"
                                            prop:value=model.status.as_str()
                                            on:change=move |event| {
                                                if let Ok(next) =
                                                    event_target_value(&event).parse::<Status>()
                                                {
                                                    on_status_change.call((id.clone(), next));
                                                }
                                            }
                                        >
                                            {Status::all().into_iter().map(|option| view! {
                                                <option value={option.as_str()}>{option.label()}</option>
                                            }).collect_view()}
                                        </select>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()
                }}
            </tbody>
        </table>
    }
}

async fn fetch_games() -> Result<Vec<Game>, String> {
    let response = Request::get(&format!("{API_BASE}/games"))
        .send()
        .await
        .map_err(|error| format!("Failed to load games: {error}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to load games: {}",
            response.status()
        ));
    }

    response
        .json::<Vec<Game>>()
        .await
        .map_err(|error| format!("Failed to parse games: {error}"))
}

async fn fetch_model_definitions() -> Result<Vec<ModelDefinition>, String> {
    let response = Request::get(&format!("{API_BASE}/model-definitions"))
        .send()
        .await
        .map_err(|error| format!("Failed to load model definitions: {error}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to load model definitions: {}",
            response.status()
        ));
    }

    response
        .json::<Vec<ModelDefinition>>()
        .await
        .map_err(|error| format!("Failed to parse model definitions: {error}"))
}

async fn fetch_user_models() -> Result<Vec<UserModelListItem>, String> {
    let response = Request::get(&format!("{API_BASE}/user-models"))
        .send()
        .await
        .map_err(|error| format!("Failed to load models: {error}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to load models: {}",
            response.status()
        ));
    }

    response
        .json::<Vec<UserModelListItem>>()
        .await
        .map_err(|error| format!("Failed to parse models: {error}"))
}

async fn create_model_definition(
    name: String,
    game_id: String,
) -> Result<ModelDefinition, String> {
    let response = Request::post(&format!("{API_BASE}/model-definitions"))
        .header("content-type", "application/json")
        .body(
            serde_json::to_string(&CreateModelDefinitionRequest { name, game_id })
                .map_err(|error| format!("Failed to serialize model definition: {error}"))?,
        )
        .send()
        .await
        .map_err(|error| format!("Failed to create model definition: {error}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to create model definition: {}",
            response.status()
        ));
    }

    response
        .json::<ModelDefinition>()
        .await
        .map_err(|error| format!("Failed to parse model definition: {error}"))
}

async fn create_user_model(
    model_definition_id: String,
    quantity: i32,
    status: Status,
) -> Result<UserModel, String> {
    let response = Request::post(&format!("{API_BASE}/user-models"))
        .header("content-type", "application/json")
        .body(
            serde_json::to_string(&CreateUserModelRequest {
                model_definition_id,
                quantity,
                status,
            })
            .map_err(|error| format!("Failed to serialize user model: {error}"))?,
        )
        .send()
        .await
        .map_err(|error| format!("Failed to create user model: {error}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to create user model: {}",
            response.status()
        ));
    }

    response
        .json::<UserModel>()
        .await
        .map_err(|error| format!("Failed to parse user model: {error}"))
}

async fn update_user_model_status(id: String, status: Status) -> Result<UserModel, String> {
    let response = Request::patch(&format!("{API_BASE}/user-models/{id}"))
        .header("content-type", "application/json")
        .body(
            serde_json::to_string(&UpdateUserModelRequest { status })
                .map_err(|error| format!("Failed to serialize status: {error}"))?,
        )
        .send()
        .await
        .map_err(|error| format!("Failed to update user model: {error}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to update user model: {}",
            response.status()
        ));
    }

    response
        .json::<UserModel>()
        .await
        .map_err(|error| format!("Failed to parse user model: {error}"))
}

#[derive(Clone, Deserialize)]
struct Game {
    id: String,
    name: String,
}

#[derive(Clone, Deserialize)]
struct ModelDefinition {
    id: String,
    name: String,
    game: Game,
}

#[derive(Clone, Deserialize)]
struct UserModelListItem {
    id: String,
    model_name: String,
    game_name: String,
    quantity: i32,
    status: Status,
}

#[derive(Clone, Deserialize)]
struct UserModel {
    id: String,
    model_definition_id: String,
    quantity: i32,
    status: Status,
}

#[derive(Serialize)]
struct CreateModelDefinitionRequest {
    name: String,
    game_id: String,
}

#[derive(Serialize)]
struct CreateUserModelRequest {
    model_definition_id: String,
    quantity: i32,
    status: Status,
}

#[derive(Serialize)]
struct UpdateUserModelRequest {
    status: Status,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Unassembled,
    Assembled,
    Painted,
}

impl Status {
    fn all() -> [Status; 3] {
        [Status::Unassembled, Status::Assembled, Status::Painted]
    }

    fn label(self) -> &'static str {
        match self {
            Status::Unassembled => "Unassembled",
            Status::Assembled => "Assembled",
            Status::Painted => "Painted",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Status::Unassembled => "unassembled",
            Status::Assembled => "assembled",
            Status::Painted => "painted",
        }
    }
}

impl std::str::FromStr for Status {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "unassembled" => Ok(Status::Unassembled),
            "assembled" => Ok(Status::Assembled),
            "painted" => Ok(Status::Painted),
            _ => Err(()),
        }
    }
}
