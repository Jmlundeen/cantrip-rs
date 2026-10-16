use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// As this creature enters, look at an opponent's hand, then choose any card name (CR 614.12a). The opponent is chosen
/// first; the name an unscripted player takes is the costliest nonland card in the hand looked at.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let you = entering.controller;
    let source = entering.object;
    let opponents: Vec<PlayerId> = g.state().opponents(you).collect();
    let Some(opponent) = choose::player(g, you, source, opponents, "LookAtHand", 0)? else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    let mut suggestion: Option<(u32, CardId)> = None;
    for &card in &hand {
        let id = g.chars(card).card;
        let def = g.card(id);
        if !def.is(CardType::Land) && suggestion.is_none_or(|(value, _)| def.cost.mana_value() > value) {
            // A split card has two names and a player names one of them (CR 709.4a): its left half's.
            let named = def.split.map_or(id, |(left, _)| g.card_id(left));
            suggestion = Some((def.cost.mana_value(), named));
        }
    }
    if let Answer::Name(name) = g.ask(you, RequestKind::ChooseName { source, looked_at: hand, suggestion: suggestion.map(|s| s.1), kind: NameKind::Any })? {
        entering.chosen_card = name;
    }
    Ok(())
}

/// Spells your opponents cast with the chosen name cost {2} more to cast. Activated abilities of sources with the
/// chosen name cost {2} more to activate unless they're mana abilities (CR 601.2f, 602.2b): the rules ask this only of
/// the abilities that use the stack.
fn tax(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    let Some(chosen) = q.obj(source).marks.chosen_card else { return CostDelta::NONE };
    let (object, spell) = match costed {
        Costed::Spell(id) => (id, true),
        Costed::Ability(id, _) => (id, false),
    };
    if !cantrip_mechanics::library::object_has_name(q, object, chosen) || (spell && player == q.controller(source)) {
        return CostDelta::NONE;
    }
    CostDelta::more(2)
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    cost_changes: &[tax],
    ..oracle::ANOINTED_PEACEKEEPER
};
