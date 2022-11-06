use serenity::{
    builder::{CreateButton, CreateComponents},
    model::prelude::component::ButtonStyle,
};

use crate::stats::Stat;

/// Build a button based on an id and display string
pub fn button(id: &str, display_name: &str) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(id);
    b.label(display_name);
    b.style(ButtonStyle::Primary);
    b
}

/// Build a set of rows containing 5 buttons each at most
pub fn buttons_from_stats<'a>(
    components: &'a mut CreateComponents,
    stats: &[Stat],
) -> &'a mut CreateComponents {
    stats.chunks(5).for_each(|chunk| {
        components.create_action_row(|row| {
            chunk.iter().for_each(|stat| {
                row.add_button(button(&stat.id, &stat.display_name));
            });
            row
        });
    });
    components
}
