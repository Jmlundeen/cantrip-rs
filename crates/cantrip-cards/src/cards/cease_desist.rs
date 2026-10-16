use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The second of "up to two target cards from a single graveyard" (CR 115.1, 601.2c): a card of the graveyard the first
/// slot's card is in, and not that card itself. When the first slot chose nothing, any card in any graveyard: one target
/// is a legal choice of "up to two", and the two slots together allow exactly the choices of zero, one or two cards that
/// all lie in one graveyard.
fn other_card_in_the_same_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !(targets::CARD_IN_A_GRAVEYARD.filter)(q, asker, t) {
        return false;
    }
    match q.chosen_targets().first() {
        Some(&Target::Object(first)) => first != id && q.obj(first).zone == q.obj(id).zone,
        _ => true,
    }
}

/// Cease: "Exile up to two target cards from a single graveyard. Target player gains 2 life and draws a card." The cards
/// still in that graveyard are exiled together (CR 608.2b: a target that became illegal is left alone and the rest of
/// the spell still happens), then the player, if still a legal target, gains 2 life and draws.
fn cease(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<ObjectId> = (0..2).filter_map(|slot| targets::object(g, item, slot)).collect();
    g.exile_all(&cards);
    if g.target_legal(item, 2)
        && let Target::Player(player) = item.targets[2]
    {
        g.gain_life(player, 2);
        g.draw(player, 1);
    }
    Ok(())
}

/// Desist: "Destroy all artifacts and enchantments." Every one on the battlefield goes at once (CR 701.8a, 101.4); an
/// artifact creature or enchantment creature too, and indestructible or regeneration is the verb's business.
fn desist(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Artifact) || c.is(CardType::Enchantment)
        })
        .collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CEASE: CardDef = CardDef {
    spell: spell(
        &[
            targets::CARD_IN_A_GRAVEYARD.optional().labelled("First card to exile (up to two, from a single graveyard)"),
            target(other_card_in_the_same_graveyard).optional().labelled("Second card to exile, from the same graveyard"),
            targets::PLAYER.labelled("Player to gain 2 life and draw a card"),
        ],
        cease,
    ),
    ..oracle::CEASE
};

pub static DESIST: CardDef = CardDef { spell: spell(&[], desist), ..oracle::DESIST };

pub static CARD: CardDef = CardDef { split: Some((&CEASE, &DESIST)), ..oracle::CEASE_DESIST };

pub static EXTRA: &[&CardDef] = &[&CEASE, &DESIST];
