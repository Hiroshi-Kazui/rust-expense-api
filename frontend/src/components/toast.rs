use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub message: String,
    pub kind: ToastKind,
}

#[component]
pub fn ToastContainer(messages: ReadSignal<Vec<ToastMessage>>) -> impl IntoView {
    let each_fn = move || messages.get().into_iter().enumerate().collect::<Vec<_>>();
    view! {
        <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
            <For
                each=each_fn
                key=|(i, _)| *i
                children=|(_, msg)| {
                    let class = match msg.kind {
                        ToastKind::Success => "bg-green-500 text-white px-4 py-3 rounded shadow-lg text-sm",
                        ToastKind::Error => "bg-red-500 text-white px-4 py-3 rounded shadow-lg text-sm",
                    };
                    view! { <div class={class}>{msg.message.clone()}</div> }
                }
            />
        </div>
    }
}

pub fn use_toast() -> (ReadSignal<Vec<ToastMessage>>, impl Fn(String, ToastKind) + Clone) {
    let (messages, set_messages) = signal(Vec::<ToastMessage>::new());

    let show_toast = move |message: String, kind: ToastKind| {
        set_messages.update(|msgs| {
            msgs.push(ToastMessage { message, kind });
        });
        let set_messages = set_messages.clone();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            set_messages.update(|msgs| {
                if !msgs.is_empty() {
                    msgs.remove(0);
                }
            });
        });
    };

    (messages, show_toast)
}
