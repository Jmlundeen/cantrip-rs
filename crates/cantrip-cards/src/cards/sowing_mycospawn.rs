use cantrip_api::*;
use cantrip_mechanics::{devoid, exile_target, targets};
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_oracle as oracle;

/// "If it was kicked" (CR 603.4): the ability does not trigger at all when the kicker was not paid.
fn cast_this_spell_kicked(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if *spell == source && q.obj(source).kicked > 0 => Fired::PLAIN,
        _ => None,
    }
}

/// Search your library for a land card, put it onto the battlefield, then shuffle (CR 701.23b: it may find nothing).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, |q, c| q.chars(c).is(CardType::Land))?;
    Ok(())
}

/// Exile target land.
fn exile_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

/// Kicker {1}{C} (CR 702.33, 107.4c).
const KICKER_COST: ManaCost = ManaCost { generic: 1, symbols: [0, 0, 0, 0, 0, 1], hybrid: [None; 6], phyrexian: [0; 5], x: 0 };

pub static CARD: CardDef = CardDef {
    colors: devoid::COLORS,
    kicker: Some(Kicker { cost: KICKER_COST, multi: false, waterbend: false }),
    abilities: &[
        triggered(cantrip_mechanics::triggers::CAST_THIS_SPELL, search, "When you cast this spell, search your library for a land card, put it onto the battlefield, then shuffle.")
            .anywhere(),
        triggered(when(EventMask::SPELL_CAST, cast_this_spell_kicked), exile_land, "When you cast this spell, if it was kicked, exile target land.")
            .targets(&[targets::LAND])
            .anywhere(),
    ],
    ..oracle::SOWING_MYCOSPAWN
};
