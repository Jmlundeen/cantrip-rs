use cantrip_api::*;
use cantrip_mechanics::{choose, destroy_target, tokens};
use cantrip_oracle as oracle;

/// +2: You may sacrifice another permanent. If you do, you gain 1 life and draw a card. The permanent is chosen as the
/// ability resolves (CR 608.2d) among those its controller controls other than Vraska; declining is an answer, and "if you do"
/// is read off whether the permanent really left.
fn sacrifice_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let others: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| id != item.object).collect();
    let Some(&permanent) = choose::cards(g, item.controller, item.object, Purpose::Sacrifice, others, 1, true)?.first() else { return Ok(()) };
    if !g.sacrifice(&[permanent]).is_empty() {
        g.gain_life(item.controller, 1);
        g.draw(item.controller, 1);
    }
    Ok(())
}

fn nonland_permanent_with_mana_value_three_or_less(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id)
        && !q.chars(id).is(CardType::Land) && q.mana_value(id) <= 3)
}

/// -3: Destroy target nonland permanent with mana value 3 or less.
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

fn that_player_loses(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.lose_game(p);
    }
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Vraska, Golgari Queen emblem",
    has_cost: false,
    abilities: &[triggered(when_asked(EventMask::DAMAGE, cantrip_mechanics::triggers::creature_you_control_deals_combat_damage_to_player), that_player_loses, "Whenever a creature you control deals combat damage to a player, that player loses the game.")],
    text: "Whenever a creature you control deals combat damage to a player, that player loses the game.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(2), sacrifice_to_draw, "+2: You may sacrifice another permanent. If you do, you gain 1 life and draw a card."),
        ability(loyalty(-3), destroy, "\u{2212}3: Destroy target nonland permanent with mana value 3 or less.")
            .targets(&[target(nonland_permanent_with_mana_value_three_or_less)]),
        ability(loyalty(-9), emblem, "\u{2212}9: You get an emblem with \"Whenever a creature you control deals combat damage to a player, that player loses the game.\""),
    ],
    ..oracle::VRASKA_GOLGARI_QUEEN
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
