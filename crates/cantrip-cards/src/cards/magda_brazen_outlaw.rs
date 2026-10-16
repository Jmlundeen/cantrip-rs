use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Other Dwarves you control".
fn other_dwarves_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    affected != source && so_far.has_subtype(q.words(source).subtype(Subtype::Dwarf)) && s.obj(affected).controller == s.obj(source).controller
}

/// "Whenever a Dwarf you control becomes tapped" (CR 701.26a, 603.2): by any path that taps a permanent.
fn dwarf_becomes_tapped(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Tapped { permanent }
            if q.controller(*permanent) == q.controller(source)
                && q.chars(*permanent).has_subtype(q.words(source).subtype(Subtype::Dwarf)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_treasure(g, item.controller)
}

/// "Search your library for an artifact or Dragon card, put that card onto the battlefield, then shuffle."
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let dragon = g.words(item.object).subtype(Subtype::Dragon);
    search_to_battlefield(g, player, item.object, Find::ONE, false, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Artifact) || chars.has_subtype(dragon)
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::TAPPED, dwarf_becomes_tapped), treasure, "Whenever a Dwarf you control becomes tapped, create a Treasure token."),
        ability(sacrifice(5, cantrip_mechanics::targets::filters::treasure), tutor, "Sacrifice five Treasures: Search your library for an artifact or Dragon card, put that card onto the battlefield, then shuffle."),
    ],
    statics: &[static_ability(Layer::PtModify, other_dwarves_you_control, cantrip_mechanics::statics::plus_pt::<1, 0>, "Other Dwarves you control get +1/+0.")],
    ..oracle::MAGDA_BRAZEN_OUTLAW
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
