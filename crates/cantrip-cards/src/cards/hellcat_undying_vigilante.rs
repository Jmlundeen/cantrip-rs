use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "When Hellcat dies, return her to the battlefield under her owner's control with a +1/+1 counter on her. She loses all
/// abilities and gains haste." The card it became in the graveyard, if it is still there (CR 400.7), enters with the counter
/// (CR 122.6). The effect is on the new object, with no duration, so it lasts as long as she stays on the battlefield
/// (CR 611.2a); in layer 6 she loses all abilities, her own haste and this trigger among them, and then gains haste, which
/// comes later (CR 613.1f, 613.7), so she does not return a second time.
fn return_changed(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    let owner = g.obj(card).owner;
    let hellcat = g.put_onto_battlefield_with_counters(card, owner, false, &[(CounterKind::PlusOne, 1)])?;
    g.add_effect(item.object, hellcat, EffectKind::LoseAllAbilities, Duration::Indefinite);
    g.add_effect(item.object, hellcat, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_DIES,
        return_changed,
        "When Hellcat dies, return her to the battlefield under her owner's control with a +1/+1 counter on her. She loses all abilities and gains haste.",
    )
    .look_back()],
    ..oracle::HELLCAT_UNDYING_VIGILANTE
};
