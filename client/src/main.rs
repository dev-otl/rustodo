use ev::MouseEvent;
use gloo_net::http::Request;
use leptos::*;
use logging::log;
use serde::{Deserialize, Serialize};
const SERVER: &'static str = "<Your server here>";

#[derive(Serialize, Deserialize, Clone)]
struct ResponseTask {
    task_id: i64,
    task_title: String,
    task_description: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Response {
    user_id: i64,
    username: String,
    tasks: Vec<ResponseTask>,
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct LoginInfo {
    username: String,
    password: String,
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! {<App />})
}

#[component]
fn App() -> impl IntoView {
    let (reload_needed, set_reload_needed) = create_signal(true);

    let (data, set_data) = create_signal(Response {
        user_id: -1,
        username: "Anon".to_string(),
        tasks: vec![],
        success: false,
        message: "SessionGetError".to_string(),
    });

    create_effect(move |_| {
        spawn_local(async move {
            if reload_needed.get() {
                let fetched_response: Response = Request::get(&format!("{}/data", SERVER))
                    .credentials(web_sys::RequestCredentials::Include)
                    .header("access-control-allow-origin", "*")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                set_data.set(fetched_response);
                set_reload_needed.set(false);
            }
        })
    });

    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());

    let on_login_info_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let login_info = LoginInfo {
            username: username.get(),
            password: password.get(),
        };
        spawn_local(async move {
            let fetched_response = Request::post(&format!("{}/login", SERVER))
                .credentials(web_sys::RequestCredentials::Include)
                .json(&login_info)
                .unwrap()
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            set_data.set(fetched_response);
        })
    };

    let on_signout = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let fetched_response = Request::delete(&format!("{}/logout", SERVER))
                .credentials(web_sys::RequestCredentials::Include)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            set_data.set(fetched_response);
        })
    };

    let (selected_task_id, set_selected_task_id) = create_signal(-1);
    let (selected_task_title, set_selected_task_title) = create_signal("".to_string());
    let (selected_task_description, set_selected_task_description) = create_signal("".to_string());

    let on_new_task_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let fetched_response: Response = Request::post(&format!("{}/task", SERVER))
                .credentials(web_sys::RequestCredentials::Include)
                .json(&ResponseTask {
                    task_id: -1,
                    task_title: selected_task_title.get(),
                    task_description: selected_task_description.get(),
                })
                .unwrap()
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            set_selected_task_id.set(-1);
            set_selected_task_title.set("".to_string());
            set_selected_task_description.set("".to_string());
            set_data.set(fetched_response);
        })
    };

    let (is_edit_mode, set_is_edit_mode) = create_signal(false);

    let on_task_edit_click = move |ev: MouseEvent| {
        ev.prevent_default();
        let task_id: i64 = event_target_value(&ev).parse().unwrap();
        if is_edit_mode.get() && task_id == selected_task_id.get() {
            spawn_local(async move {
                let fetched_response: Response = Request::put(&format!("{}/task", SERVER))
                    .credentials(web_sys::RequestCredentials::Include)
                    .json(&ResponseTask {
                        task_id: selected_task_id.get(),
                        task_title: selected_task_title.get(),
                        task_description: selected_task_description.get(),
                    })
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                set_data.set(fetched_response);
                set_selected_task_id.set(-1);
                set_selected_task_title.set("".to_string());
                set_selected_task_description.set("".to_string());
                set_is_edit_mode.set(false);
            })
        } else {
            set_selected_task_id.set(task_id);
            for task in data.get().tasks {
                if task.task_id == task_id {
                    set_selected_task_title.set(task.task_title);
                    set_selected_task_description.set(task.task_description);
                } else {
                }
            }
            set_is_edit_mode.set(true);
        }
    };

    let on_task_delete_click = move |ev: MouseEvent| {
        ev.prevent_default();
        let task_id: i64 = event_target_value(&ev).parse().unwrap();
        if is_edit_mode.get() {
            set_selected_task_id.set(-1);
            set_selected_task_title.set("".to_string());
            set_selected_task_description.set("".to_string());
            set_is_edit_mode.set(false);
        } else {
            spawn_local(async move {
                let fetched_response: Response = Request::delete(&format!("{}/task", SERVER))
                    .credentials(web_sys::RequestCredentials::Include)
                    .json(&ResponseTask {
                        task_id: task_id,
                        task_title: "".to_string(),
                        task_description: "".to_string(),
                    })
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                set_data.set(fetched_response);
            })
        }
    };

    view! {
        <>
        <div class="d-flex flex-row justify-content-between min-vh-100">
            <div class="d-flex flex-column flex-shrink border">
                <div class="d-flex flex-row justify-content-between">
                    <div class="d-flex m-2 p-1">
                        {move || format!("Hi {}!", data.get().username)}
                    </div>
                    <button class="btn btn-light m-2 p-2 " on:click={on_signout}>"Sign out"</button>
                </div>

                <form class="d-flex flex-column form" on:submit=on_login_info_submit>
                    <div>
                        <input placeholder="Username" class="p-2 m-2" type="text" on:input=move |ev| {
                            set_username.set(event_target_value(&ev)) } prop:value=move||username.get() />
                    </div>
                    <div>
                        <input placeholder="Password" class="p-2 m-2" type="text" on:input=move |ev| {
                            set_password.set(event_target_value(&ev)) } prop:value=move|| password.get() />
                    </div>
                    <div class="d-flex flex-row justify-content-end">
                        <input class="btn btn-light m-2 p-2" type="submit" value="Sign in" />
                        <input class="btn btn-light m-2 p-2" type="reset" value="Clear" />
                    </div>
                </form>
            </div>
            <div class="d-flex flex-column flex-fill justify-content-top align-items-center flex-fill">
                <div class="h1 d-flex flex-row m-2 p-2"><u>"Your To Dos"</u></div>
                <For each=move || data.get().tasks key=|task| task.task_id children=move | task:ResponseTask| { view! {
                    <form class="d-flex flex-column form bg-light rounded p-2 m-2">
                    //<div>{task.task_id}</div>

                    <div>
                        <input class="text text-center p-2 m-2" type="text" on:input=move |ev| {
                            set_selected_task_title.set(event_target_value(&ev)) } disabled=move|| selected_task_id.get()
                            !=task.task_id prop:value={ if selected_task_id.get() !=task.task_id { task.task_title } else {
                            selected_task_title.get() }} />
                    </div>
                    <div>
                        <input class="text text-center p-2 m-2" type="text" on:input=move |ev| {
                            set_selected_task_description.set(event_target_value(&ev)) } disabled=move||
                            selected_task_id.get() !=task.task_id prop:value=if selected_task_id.get() !=task.task_id {
                            task.task_description } else { selected_task_description.get() } />
                    </div>
                    <div class="d-flex flex-row justify-content-end">
                        <button class="btn btn-light m-2 p-2" value={task.task_id} prop:value=move || task.task_id
                            on:click=on_task_edit_click>{move|| if
                            selected_task_id.get() != task.task_id{"Edit"} else {"Done"}}</button>
                        <button class="btn btn-light m-2 p-2" on:click=on_task_delete_click prop:value=move ||
                            task.task_id>{move|| if
                            selected_task_id.get() != task.task_id{"Delete"} else {"Cancel"}}</button>
                    </div>
                    </form>
                    }
                    }
                    />
                    <form class="d-flex flex-column form bg-light rounded p-2 m-4 " on:submit=on_new_task_submit>
                        //<div>"New Task"</div>
                        <div>
                            <input class="text text-center p-2 m-2" type="text" disabled=move|| selected_task_id.get()!=-1
                                on:input=move |ev| { set_selected_task_title.set(event_target_value(&ev)) } prop:value=move
                                || if selected_task_id.get() !=1 { "" .to_string() } else { selected_task_title.get() } />
                        </div>
                        <div>
                            <input class="text text-center p-2 m-2" type="text" disabled=move|| selected_task_id.get()!=-1
                                on:input=move |ev| { set_selected_task_description.set(event_target_value(&ev)) }
                                prop:value=move ||if selected_task_id.get() !=1 { "" .to_string() } else {
                                selected_task_description.get() } />
                        </div>
                        <div class="d-flex flex-row justify-content-end">
                            <button class="btn btn-light m-2 p-2" disabled=move|| selected_task_id.get()!=-1
                                type="submit">"Add"</button>
                            <button class="btn btn-light m-2 p-2" disabled=move|| selected_task_id.get()!=-1
                                type="reset">"Clear"</button>
                        </div>
                    </form>
                    <div>{move || serde_json::to_string(&data)}</div>
            </div>
        </div>
    </>
    }
}
