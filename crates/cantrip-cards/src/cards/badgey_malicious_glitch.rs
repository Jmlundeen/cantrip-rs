use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever Badgey becomes tapped" (CR 603.2).
fn when_badgey_becomes_tapped(
    _: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::Tapped { permanent } = event else { return None };
    let id = *permanent;
    let subject = Target::Object(id);
    if asker.source == id {
        return Fired::about(subject);
    }
    None
}

/// "Whenever Badgey becomes tapped, each opponent loses 1 life and you gain 1 life." (CR 608.2, 119.3).
fn opponent_loses_1_life_gain(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.end_sentence();
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::TAPPED, when_badgey_becomes_tapped),
            opponent_loses_1_life_gain,
            "Whenever Badgey becomes tapped, each opponent loses 1 life and you gain 1 life.",
        ),
    ],
    ..oracle::BADGEY_MALICIOUS_GLITCH
};
