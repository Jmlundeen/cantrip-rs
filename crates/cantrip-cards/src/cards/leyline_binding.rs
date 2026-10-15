use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// Domain (an ability word): this spell costs {1} less to cast for each basic land type among lands you
/// control.
fn domain(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed) -> i8 {
    if !matches!(costed, Costed::Spell(_)) {
        return 0;
    }
    let lands: Vec<Chars> = q.permanents_of(player).into_iter().map(|id| q.chars(id)).filter(|c| c.is(CardType::Land)).collect();
    let basic = [Subtype::Plains, Subtype::Island, Subtype::Swamp, Subtype::Mountain, Subtype::Forest];
    -(basic.iter().filter(|&&t| lands.iter().any(|c| c.subtypes.has(t))).count() as i8)
}

/// "Exile target nonland permanent an opponent controls until this enchantment leaves the battlefield" (CR 610.3).
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.exile_with(&[permanent], item.object, true);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[domain],
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this enchantment enters, exile target nonland permanent an opponent controls until this enchantment leaves the battlefield.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS],
        effect: exile_until_it_leaves,
        ..TriggeredDef::BLANK
    })],
    ..oracle::LEYLINE_BINDING
};
