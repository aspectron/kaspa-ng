use crate::imports::*;

#[derive(Debug)]
pub struct PaginationPage {
    pub page: u64,
    pub skip: u64,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct PaginationOptions {
    pub first: String,
    pub last: String,
    pub prev: String,
    pub next: String,
}
impl PaginationOptions {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            first: i18n("FIRST").to_string(),
            last: i18n("LAST").to_string(),
            prev: i18n("PREV").to_string(),
            next: i18n("NEXT").to_string(),
        }
    }
}

pub struct Pagination {
    // pub name: Option<String>,
    pub total_pages: u64,
    pub active_page: u64,
    pub is_last: bool,
    pub is_first: bool,
    pub prev: u64,
    pub next: u64,
    pub last: u64,
    pub last_skip: u64,
    pub prev_skip: u64,
    pub next_skip: u64,
    pub total: u64,
    pub skip: u64,
    pub limit: u64,
    pub pages: Arc<Vec<PaginationPage>>,
    pub max_pages: u64,
    pub half: u64,
    pub btn_size1: Vec2,
    pub btn_size2: Vec2,
    //pub callback: Arc<Mutex<Option<PaginationCallback>>>,
    pub options: Option<PaginationOptions>,
}

impl core::fmt::Debug for Pagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Pagination")
            // .field("name", &self.name)
            .field("total_pages", &self.total_pages)
            .field("active_page", &self.active_page)
            .field("is_last", &self.is_last)
            .field("is_first", &self.is_first)
            .field("prev", &self.prev)
            .field("next", &self.next)
            .field("last", &self.last)
            .field("last_skip", &self.last_skip)
            .field("prev_skip", &self.prev_skip)
            .field("next_skip", &self.next_skip)
            .field("total", &self.total)
            .field("skip", &self.skip)
            .field("limit", &self.limit)
            .field("pages", &self.pages)
            .field("max_pages", &self.max_pages)
            .field("half", &self.half)
            .field("options", &self.options);
        Ok(())
    }
}

impl Pagination {
    pub fn new(total: u64, skip: Option<u64>, limit: Option<u64>, max_pages: Option<u64>) -> Self {
        let skip = skip.unwrap_or(0);
        let limit = limit.unwrap_or(25);
        let total_pages = (total as f32 / limit as f32).ceil() as u64;
        let active_page = total_pages.min(((skip + 1) as f64 / limit as f64).ceil() as u64);
        let max_pages = max_pages.unwrap_or(10).min(total_pages).min(10);
        let half = (max_pages as f64 / 2.0).floor() as u64;
        let prev = 1.max(active_page.saturating_sub(1));
        let next = total_pages.min(active_page + 1);
        let mut page = 1;
        // log_info!(
        //     "active_page: {active_page}, half:{half}, max_pages:{max_pages}, total_pages:{total_pages}"
        // );
        if active_page > half {
            page = active_page + half.min(total_pages - active_page) + 1 - max_pages;
        }

        let mut pages = Vec::new();
        for _ in 0..max_pages {
            pages.push(PaginationPage {
                page,
                skip: (page - 1) * limit,
                active: active_page == page,
            });
            page += 1;
        }
        Self {
            //name: None,
            total_pages,
            active_page,
            is_last: active_page == total_pages,
            is_first: active_page < 2,
            prev,
            next,
            last: total_pages,
            last_skip: total_pages.saturating_sub(1) * limit,
            prev_skip: prev.saturating_sub(1) * limit,
            next_skip: next.saturating_sub(1) * limit,
            total,
            skip,
            limit,
            pages: Arc::new(pages),
            max_pages,
            half,
            btn_size1: Vec2::new(30_f32, 30_f32), // numbers
            btn_size2: Vec2::new(50_f32, 30_f32), // first,prev,next,last
            options: None,
        }
    }

    pub fn with_options(mut self, options: PaginationOptions) -> Result<Self> {
        self.options = Some(options);
        Ok(self)
    }

    fn calculate_padding(&self, ui: &mut Ui, options: &PaginationOptions) -> f32 {
        let available_width = ui.available_width();
        let mut total_width = ui.spacing().item_spacing.x;

        let btns = vec![&options.first, &options.last, &options.last, &options.last];
        let btn_margin = ui.spacing().item_spacing.x;
        let btn_padding = ui.spacing().button_padding.x * 2.0;
        let pages = self.pages.clone();
        for p in pages.iter() {
            let g = WidgetText::from(p.page.to_string()).into_galley(
                ui,
                None,
                available_width,
                TextStyle::Button,
            );
            let button_width = (g.size().x + btn_padding).max(self.btn_size1.x);
            total_width += button_width + btn_margin;
        }

        for btn in btns {
            let g = WidgetText::from(btn).into_galley(ui, None, available_width, TextStyle::Button);
            let button_width = (g.size().x + btn_padding).max(self.btn_size2.x);
            total_width += button_width + btn_margin;
        }

        let padding = (available_width - total_width) / 2.0;

        padding.max(0.0)
    }

    pub fn render(&self, ui: &mut Ui) -> Option<u64> {
        let pages = self.pages.clone();
        let is_first = self.is_first;
        let is_last = self.is_last;
        let prev_skip = self.prev_skip;
        let next_skip = self.next_skip;
        let last_skip = self.last_skip;
        let btn_size1 = self.btn_size1;
        let btn_size2 = self.btn_size2;
        // let name = self.name.clone().unwrap_or_else(|| "kng".to_string());

        let options = self.options.clone().unwrap_or_default();
        let padding = self.calculate_padding(ui, &options);

        let first_text = options.first;
        let last_text = options.last;
        let prev_text = options.prev;
        let next_text = options.next;
        let mut start = None;

        //ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.horizontal(|ui| {
            ui.add_space(padding);

            if add_btn(ui, !is_first, prev_text, btn_size2).clicked() {
                start = Some(prev_skip);
            }

            if add_btn(ui, !is_first, first_text, btn_size2).clicked() {
                start = Some(0);
            }

            for page in pages.iter() {
                if add_num_btn(ui, page.active, page.page.to_string(), btn_size1).clicked() {
                    start = Some(page.skip);
                }
            }

            if add_btn(ui, !is_last, last_text, btn_size2).clicked() {
                start = Some(last_skip);
            }

            if add_btn(ui, !is_last, next_text, btn_size2).clicked() {
                start = Some(next_skip);
            }
        });
        //});

        start
    }
}

fn add_btn(ui: &mut Ui, enabled: bool, text: impl Into<WidgetText>, min_size: Vec2) -> Response {
    ui.add_enabled(enabled, Button::new(text).min_size(min_size))
}

fn add_num_btn(ui: &mut Ui, active: bool, text: impl Into<WidgetText>, min_size: Vec2) -> Response {
    ui.add_enabled(
        !active,
        Button::new(text)
            .corner_radius(ui.visuals().widgets.hovered.corner_radius)
            .selected(active)
            .min_size(min_size),
    )
}
