use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

fn three_of_one_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 3);
    Ok(())
}

/// "Whenever one or more creatures an opponent controls attack you and aren't blocked" (CR 509.1h): one trigger for
/// the whole combat, however many of them there were.
fn opponent_attacks_you_unblocked(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Unblocked { attacker, defender }
            if *defender == Target::Player(q.controller(source)) && q.controller(*attacker) != q.controller(source) =>
        {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

/// "That player draws three cards and gains control of this artifact. Untap it." That player is whoever attacked,
/// which is always the active player: only they declare attackers (CR 508.1). The draws happen whether or not the Jewel
/// is still there to hand over; the handover and the untap do nothing for a Jewel that has left the battlefield
/// (CR 608.2a).
fn claim(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let attacker = g.state().turn.active;
    g.draw(attacker, 3);
    let jewel = item.object;
    g.gain_control(jewel, jewel, attacker, Duration::Indefinite);
    g.untap(jewel);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add three mana of any one color.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(three_of_one_color),
            any_color: true,
            adds_any: 3,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this artifact enters, draw three cards.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: cantrip_mechanics::effects::draw::<3>,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever one or more creatures an opponent controls attack you and aren't blocked, that player draws three cards and gains control of this artifact. Untap it.",
            listens: EventMask::UNBLOCKED,
            matches: opponent_attacks_you_unblocked,
            effect: claim,
            once_per_batch: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::COVETED_JEWEL
};
