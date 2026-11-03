use cantrip_api::*;
use cantrip_mechanics::{effects, prevention, targets, tokens};
use cantrip_oracle as oracle;

/// "+1: Until your next turn, prevent all damage target permanent would deal." (CR 608.2, 611.2a): nothing if the
/// target is no longer legal (CR 608.2b).
fn next_prevent_all_damage_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(
        item.object,
        permanent,
        EffectKind::Prevention { card: item.ability.card, index: 0, to: Target::None, shield: Shield::All },
        Duration::UntilNextTurnOf(item.controller),
    );
    Ok(())
}

/// "0: Until end of turn, Gideon becomes a 4/4 Human Soldier creature with indestructible that's still a planeswalker.
/// Prevent all damage that would be dealt to him this turn." (CR 611.2a, 615.1).
fn gideon_becomes_4_4_human(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let keywords = Keywords::of(&[Keyword::Indestructible]);
    effects::planeswalker_becomes_creature_preventing_damage(g, item, (4, 4), None, &[Subtype::Human, Subtype::Soldier], keywords, 1);
    Ok(())
}

/// "As long as you control a Gideon planeswalker" (CR 604.1): the emblem's owner controls a permanent that is a Gideon
/// planeswalker, Gideon read through the words of the emblem's source (CR 612).
fn controls_a_gideon_planeswalker(q: &dyn Query, asker: &Asker) -> bool {
    let gideon = q.words(asker.source).subtype(Subtype::Gideon);
    q.state().battlefield().iter().any(|&id| {
        q.controller(id) == asker.controller && q.chars(id).has_subtype(gideon) && q.chars(id).is(CardType::Planeswalker)
    })
}

/// "...you can't lose the game" (CR 104.3, 604.1).
fn emblem_cant_lose(q: &dyn Query, asker: &Asker, player: PlayerId, _: LossReason) -> bool {
    player == asker.controller && controls_a_gideon_planeswalker(q, asker)
}

/// "...and your opponents can't win the game" (CR 104.2, 604.1).
fn emblem_opponents_cant_win(q: &dyn Query, asker: &Asker, player: PlayerId) -> bool {
    q.state().opponents(asker.controller).any(|p| p == player) && controls_a_gideon_planeswalker(q, asker)
}

pub static EMBLEM: CardDef = CardDef {
    name: "Gideon of the Trials emblem",
    has_cost: false,
    text: "As long as you control a Gideon planeswalker, you can't lose the game and your opponents can't win the game.",
    loss_rules: &[emblem_cant_lose],
    win_rules: &[emblem_opponents_cant_win],
    ..CardDef::BLANK
};

/// "0: You get an emblem with "As long as you control a Gideon planeswalker, you can't lose the game and your opponents can't win the game."" (CR 114.2).
fn get_emblem_as_long_as(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                loyalty(1),
                next_prevent_all_damage_target,
                "+1: Until your next turn, prevent all damage target permanent would deal.",
            )
            .targets(&[targets::PERMANENT]),
        ability(
                loyalty(0),
                gideon_becomes_4_4_human,
                "0: Until end of turn, Gideon becomes a 4/4 Human Soldier creature with indestructible that's still a planeswalker. Prevent all damage that would be dealt to him this turn.",
            ),
        ability(
                loyalty(0),
                get_emblem_as_long_as,
                "0: You get an emblem with \"As long as you control a Gideon planeswalker, you can't lose the game and your opponents can't win the game.\"",
            ),
    ],
    granted_preventions: &[
        prevention::DAMAGE_DEALT_BY.text("Until your next turn, prevent all damage target permanent would deal."),
        prevention::ANY_DAMAGE.text("Prevent all damage that would be dealt to him this turn."),
    ],
    ..oracle::GIDEON_OF_THE_TRIALS
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
