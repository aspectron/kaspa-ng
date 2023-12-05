use crate::imports::*;

pub fn format_duration(millis: u64) -> String {
    let seconds = millis / 1000;
    // let seconds = seconds_f64 as u64;
    let days = seconds / (24 * 60 * 60);
    let hours = (seconds / (60 * 60)) % 24;
    let minutes = (seconds / 60) % 60;
    let seconds = (seconds % 60) as f64 + (millis % 1000) as f64 / 1000.0;

    if days > 0 {
        format!("{} days {:02}:{:02}:{:02.4}", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{:02}:{:02}:{:02.4}", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{:02}:{:02.4}", minutes, seconds)
    } else if millis > 1000 {
        format!("{:2.4} sec", seconds)
    } else {
        format!("{} msec", millis)
    }
}

pub fn format_address(address: &Address, range: Option<usize>) -> String {
    let address = address.to_string();

    let parts = address.split(':').collect::<Vec<&str>>();
    let prefix = parts[0];
    let payload = parts[1];
    let range = range.unwrap_or(6);
    let start = range;
    let finish = payload.len() - range;

    let left = &payload[0..start];
    // let center = style(&payload[start..finish]).dim();
    let right = &payload[finish..];

    format!("{prefix}:{left}....{right}")
}

/// SOMPI (u64) to KASPA (string) with suffix layout job generator
pub fn s2kws_layout_job(sompi : u64, network_type: &NetworkType, color : Color32, font : FontId) -> LayoutJob {

    let suffix = kaspa_suffix(network_type);
    let style = Style::default();
    
    let mut layout_job = LayoutJob::default();
    if sompi == 0 {
        let transparent = color.gamma_multiply(0.25);
        let left = RichText::new("0.0").color(color).font(font.clone());
        let right = RichText::new("0000000 ").color(transparent).font(font.clone());
        let suffix = RichText::new(suffix).color(color).font(font);
        left.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
        right.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
        suffix.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
        layout_job
    } else {
        
        let transparent = color.gamma_multiply(0.05);
        let kas = sompi_to_kaspa_string_with_trailing_zeroes(sompi);
        let mut digits = kas.chars().rev().take_while(|c| *c == '0').count();
        if digits == 8 {
            digits = 7;
        }
        let (left, right) = kas.split_at(kas.len() - digits);
        let right = right.to_string() + " ";

        let left = RichText::new(left).color(color).font(font.clone());
        let right = RichText::new(right).color(transparent).font(font.clone());
        let suffix = RichText::new(suffix).color(color).font(font);
        left.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
        right.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
        suffix.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
        layout_job
    }

}
