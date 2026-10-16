use cantrip_api::*;
use cantrip_mechanics::statics::itself;
use cantrip_mechanics::triggers::{annihilator, CAST_THIS_SPELL};
use cantrip_oracle as oracle;

/// "When you cast this spell, take an extra turn after this one." Triggers from the stack (CR 603.2, 702.85a's shape).
fn extra_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

/// "When Emrakul is put into a graveyard from anywhere" (CR 113.6k): the card is in the graveyard it was put into.
fn put_into_graveyard(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Graveyard) => Fired::PLAIN,
        _ => None,
    }
}

/// "Its owner shuffles their graveyard into their library."
fn shuffle_graveyard_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let owner = g.obj(item.object).owner;
    let graveyard = g.state().zone(Zone::graveyard(owner)).to_vec();
    g.put_into_library(&graveyard, LibraryPlace::Top);
    g.shuffle(owner);
    Ok(())
}

/// "Protection from spells that are one or more colors."
fn protection(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protection_colored_spells = true;
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CAST_THIS_SPELL, extra_turn, "When you cast this spell, take an extra turn after this one.")
            .anywhere(),
        annihilator::<6>("Annihilator 6 (Whenever this creature attacks, defending player sacrifices six permanents.)"),
        triggered(when(EventMask::ZONE_CHANGE, put_into_graveyard), shuffle_graveyard_in, "When Emrakul is put into a graveyard from anywhere, its owner shuffles their graveyard into their library.")
            .anywhere(),
    ],
    statics: &[static_ability(Layer::Ability, itself, protection, "Protection from spells that are one or more colors")],
    ..oracle::EMRAKUL_THE_AEONS_TORN
};
