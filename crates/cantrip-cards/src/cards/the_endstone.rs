use cantrip_api::*;
use cantrip_mechanics::triggers::{self, YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "Whenever you play a land or cast a spell" (CR 305.1, 601.2a, 603.2): a land its controller played as the special action (a
/// land put onto the battlefield by an effect is not played), or a spell they cast (a copy that is not cast does not count).
fn you_play_a_land_or_cast_a_spell(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, reason: ZoneReason::PlayLand, .. } if to.is(ZoneKind::Battlefield) && q.obj(*new).controller == me.controller => {
            Fired::PLAIN
        }
        _ => triggers::you_cast_a_spell(q, me, event),
    }
}

fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "Your life total becomes half your starting life total, rounded up" (CR 119.5, 107.1a): gaining or losing the difference.
fn halve_to_starting_half(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let start = g.state().player(item.controller).starting_life as u32;
    g.set_life_total(item.controller, start.div_ceil(2) as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask(EventMask::SPELL_CAST.0 | EventMask::ZONE_CHANGE.0), you_play_a_land_or_cast_a_spell),
            draw,
            "Whenever you play a land or cast a spell, draw a card.",
        ),
        triggered(
            YOUR_END_STEP,
            halve_to_starting_half,
            "At the beginning of your end step, your life total becomes half your starting life total, rounded up.",
        ),
    ],
    ..oracle::THE_ENDSTONE
};
