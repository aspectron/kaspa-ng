use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};

trait BindingT: Send + Sync + 'static {
    fn render(&self) -> bool;
}

pub struct Binding<FnRender, FnHandler, V>
where
    FnRender: Fn() -> Option<V> + Send + Sync + 'static,
    FnHandler: Fn(V) + Send + Sync + 'static,
{
    render: FnRender,
    handler: FnHandler,
}

impl<FnRender, FnHandler, V> BindingT for Binding<FnRender, FnHandler, V>
where
    FnRender: Fn() -> Option<V> + Send + Sync + 'static,
    FnHandler: Fn(V) + Send + Sync + 'static,
    V: 'static,
{
    fn render(&self) -> bool {
        if let Some(resp) = (self.render)() {
            (self.handler)(resp);
            true
        } else {
            false
        }
    }
}

pub struct Prompt {
    secret: String,
    callback: Option<Arc<dyn Fn(Secret) + Send + Sync + 'static>>,
    binding: Option<Arc<dyn BindingT + Send + Sync>>,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            secret: String::new(),
            callback: None,
            binding: None,
        }
    }

    pub fn with_secret(&mut self, callback: impl Fn(Secret) + Send + Sync + 'static) {
        self.callback = Some(Arc::new(callback));
    }

    pub fn cascade<FnRender, FnHandler, V>(
        &mut self,
        render: impl Fn() -> Option<V> + Send + Sync + 'static,
        handler: impl Fn(V) + Send + Sync + 'static,
    ) where
        FnRender: Fn() -> V + Send + Sync + 'static,
        FnHandler: Fn(V) + Send + Sync + 'static,
        V: 'static,
    {
        let binding = Binding { render, handler };

        let binding: Arc<dyn BindingT + Send + Sync + 'static> = Arc::new(binding);

        self.binding = Some(binding);
    }

    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        // if let Some(binding) = &self.binding {
        //     if binding.render() {
        //         self.binding = None
        //     }
        // }

        if self.callback.is_some() {
            egui::Window::new("Please enter your password")
                .collapsible(false)
                .show(ctx, |ui| {
                    if let Some(secret) = self.render_secret_request(ui) {
                        (self.callback.take().unwrap())(secret.clone());
                    }
                });

            true
        } else {
            false
        }
    }

    fn render_secret_request(&mut self, ui: &mut Ui) -> Option<Secret> {
        let size = egui::Vec2::new(200_f32, 40_f32);

        let message = Some("Please enter you secret TEST:".to_string());
        if let Some(message) = &message {
            ui.label(" ");
            ui.label(egui::RichText::new(message).color(egui::Color32::from_rgb(255, 128, 128)));
            ui.label(" ");
        }

        ui.label(" ");
        ui.label(" ");

        ui.add_sized(
            size,
            egui::TextEdit::singleline(&mut self.secret)
                .hint_text("Enter Password...")
                .password(true)
                .vertical_align(egui::Align::Center),
        );

        // ui.add_sized(egui::Vec2::new(120_f32,40_f32), egui::Button::new("Testing 123"));

        if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
            println!("secret: {}", self.secret);
            let secret = kaspa_wallet_core::secret::Secret::new(self.secret.as_bytes().to_vec());
            self.secret.zeroize();
            Some(secret)
        } else {
            None
        }
    }
}

pub fn with_secret(callback: impl Fn(Secret) + Send + Sync + 'static) {
    prompt().with_secret(callback);
    // self.callback = Some(Arc::new(callback));
}

pub fn cascade<FnRender, FnHandler, V>(
    render: impl Fn() -> Option<V> + Send + Sync + 'static,
    handler: impl Fn(V) + Send + Sync + 'static,
) where
    FnRender: Fn() -> V + Send + Sync + 'static,
    FnHandler: Fn(V) + Send + Sync + 'static,
    V: 'static,
{
    prompt().cascade::<FnRender, FnHandler, V>(render, handler);
}

static mut PROMPT: Option<Prompt> = None;

pub fn prompt() -> &'static mut Prompt {
    unsafe {
        if PROMPT.is_none() {
            PROMPT = Some(Prompt::new());
        }
        PROMPT.as_mut().unwrap()
    }
}
