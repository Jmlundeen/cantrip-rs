use cantrip_api::*;
use cantrip_mechanics::tokens::DRAGON_5_5;
use cantrip_oracle as oracle;

/// "If one or more artifact tokens would be created under your control" (CR 111.1, 614.1a): whose they would be, and
/// what they would be as created - a copy exception that makes one an artifact counts (CR 707.9b).
fn artifact_tokens_under_your_control(q: &dyn Query, me: &Asker, p: &Proposal) -> bool {
    matches!(p, Proposal::TokenCreation(t) if t.controller == me.controller && q.token_chars(t.token, t.copy).is(CardType::Artifact))
}

/// "... that many 5/5 red Dragon creature tokens with flying are created instead": the same count, of a token that is
/// no copy of anything.
fn dragons_instead(q: &dyn Query, _: &Asker, p: &mut Proposal) {
    if let Proposal::TokenCreation(t) = p {
        t.token = q.card_id(&DRAGON_5_5);
        t.copy = None;
    }
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(
        ReplacementKind::TokenCreation,
        artifact_tokens_under_your_control,
        dragons_instead,
        "creates 5/5 red Dragon creature tokens with flying instead",
    )],
    ..oracle::DRACONIC_VISITOR
};

pub static EXTRA: &[&CardDef] = &[&DRAGON_5_5];
