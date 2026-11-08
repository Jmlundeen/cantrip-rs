use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Untap all artifacts you control during each other player's untap step" (CR 502.3, 611.3a): every artifact its
/// controller controls untaps in each opponent's untap step, as that player's permanents untap. Two Clocks untap an
/// artifact once, not twice, and an effect that says it does not untap during its controller's untap step does not stop
/// it here (the ruling): the step being run is another player's.
fn artifacts_you_control(q: &dyn Query, asker: &Asker, object: ObjectId) -> bool {
    q.chars(object).is(CardType::Artifact) && q.obj(object).controller == asker.controller
}

pub static CARD: CardDef = CardDef {
    untap_in_others_untap_steps: &[artifacts_you_control],
    ..oracle::UNWINDING_CLOCK
};
