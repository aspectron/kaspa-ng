//use {Widget, Response, Ui, Button};
use egui::*;

pub struct Container{
    rect: Option<Rect>,
    hovered: bool,
    render_body: Box<dyn FnMut(&mut egui::Ui)>
}

impl Container{
    pub fn new(body: impl FnMut(&mut egui::Ui)+'static)->Self{
        Self { rect:None, hovered: false, render_body: Box::new(body)}
    }
}

impl Container{
    fn render(&mut self, ui: &mut Ui) -> Response{
        //ui.child_ui_with_id_source(max_rect, layout, id_source);
        let mut ui_rect = ui.available_rect_before_wrap();
        let padding = 4.0;
        ui_rect.min.x += padding;
        ui_rect.min.y += padding;
        ui_rect.max.x -= padding;
        ui_rect.max.y -= padding;

        let mut child_ui = ui.child_ui(ui_rect, Layout::top_down(Align::Min));
        
        (self.render_body)(&mut child_ui);

        let mut rect = child_ui.min_rect();
        rect.min.x -= padding;
        rect.min.y -= padding;
        rect.max.x += padding;
        rect.max.y += padding;

        self.rect = Some(rect);

        //let mut child_ui2 = ui.child_ui(ui_rect, Layout::top_down(Align::Min));

        let response = ui.interact(rect, child_ui.id(), Sense::click());
        //let response = ui.allocate_rect(rect, Sense::click());
        self.hovered = !response.clicked() && response.hovered();

        
        let painter = child_ui.painter();

        if self.hovered && self.rect.is_some(){
            let rect = self.rect.as_ref().unwrap();
            painter.rect(*rect, 2.3, Color32::BLUE, (1.0, Color32::BLUE));
            //painter.rect_stroke(*rect, 1.0, (1.0, Color32::GREEN));
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }else if let Some(rect) = self.rect.as_ref() {
            painter.rect(*rect, 2.3, Color32::LIGHT_BLUE, (1.0, Color32::LIGHT_BLUE));
            //painter.rect_stroke(*rect, 1.0, (1.0, Color32::GRAY));
        }
        
        let mut child_ui = ui.child_ui(ui_rect, Layout::top_down(Align::Min));
        (self.render_body)(&mut child_ui);


        //if let Some(cursor) = ui.visuals().interact_cursor {
        //if !clicked && response.hovered {
        //    rect.max.y += 1.0;
        ui.advance_cursor_after_rect(rect);
        //let response = ui.interact(rect, child_ui.id(), Sense::click());
        // let clicked = response.clicked();

        //let response = ui.allocate_rect(rect, Sense::click());

        response
    }
}

impl Widget for Container{
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.render(ui)
    }
}

// impl Widget for &mut Container{
//     fn ui(self, ui: &mut Ui) -> Response {
//         //ui.child_ui_with_id_source(max_rect, layout, id_source);
//         let mut ui_rect = ui.available_rect_before_wrap();
//         let padding = 4.0;
//         ui_rect.min.x += padding;
//         ui_rect.min.y += padding;
//         ui_rect.max.x -= padding;
//         ui_rect.max.y -= padding;

//         let mut child_ui = ui.child_ui(ui_rect, Layout::top_down(Align::Min));
//         let painter = child_ui.painter();
    
//         if self.hovered && self.rect.is_some(){
//             let rect = self.rect.as_ref().unwrap();
//             painter.rect(*rect, 2.3, Color32::BLUE, (1.0, Color32::BLUE));
//             //painter.rect_stroke(*rect, 1.0, (1.0, Color32::GREEN));
//             ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
//         }else if let Some(rect) = self.rect.as_ref() {
//             painter.rect(*rect, 2.3, Color32::LIGHT_BLUE, (1.0, Color32::LIGHT_BLUE));
//             //painter.rect_stroke(*rect, 1.0, (1.0, Color32::GRAY));
//         }
//         // if child_ui.add(Button::new("Hello")).clicked(){
            
//         // }
//         // if child_ui.add(Button::new("Button 2")).clicked(){
//         //     child_ui.painter().rect_filled(child_ui.min_rect(), 2.3, Color32::BLUE);
//         // }

//         (self.render_body)(&mut child_ui);

//         let mut rect = child_ui.min_rect();
//         rect.min.x -= padding;
//         rect.min.y -= padding;
//         rect.max.x += padding;
//         rect.max.y += padding;

//         self.rect = Some(rect);
//         let response = ui.allocate_rect(rect, Sense::click());//vec2(200.0, 100.0), Sense::click());
//         let clicked = response.clicked();
//         // if clicked {
//         //     child_ui.painter().rect_filled(response.rect, 2.3, Color32::GREEN);
//         //     child_ui.painter().rect_stroke(response.rect, 1.0, (1.0, Color32::BLUE));
//         // }else{
//         //     child_ui.painter().rect_stroke(response.rect, 1.0, (2.0, Color32::RED));
//         // }

        

//         //if let Some(cursor) = ui.visuals().interact_cursor {
//             if !clicked && response.hovered {
//                 self.hovered = true;
//                 //ui.ctx().set_cursor_icon(cursor);
//                 // child_ui.painter().rect(response.rect, 2.3, Color32::BLUE, (1.0, Color32::YELLOW));
//                 //child_ui.painter().rect_stroke(response.rect, 1.0, (1.0, Color32::GREEN));
//             }else{
//                 self.hovered = false;
//             }
//         //}
//         response
//     }
// }