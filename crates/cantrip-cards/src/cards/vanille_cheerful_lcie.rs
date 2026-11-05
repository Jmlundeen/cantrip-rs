use cantrip_api::*;
use cantrip_mechanics::{choose, select, targets, triggers};
use cantrip_oracle as oracle;

const FANG: &str = "Fang, Fearless l'Cie";

/// "Mill two cards, then return a permanent card from your graveyard to your hand." (CR 701.17, 608.2c, 110.4a): the
/// graveyard is read after the mill, so a card just milled may be returned. The card does not target, and returning one
/// is not optional: a permanent card is chosen whenever the graveyard holds one.
fn mill_then_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 2);
    g.end_sentence();
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let rule = select::Selection::of(select::as_many_as_can(1), &[select::PERMANENT_CARD]).called("permanent");
    let chosen = choose::select(g, you, item.object, Purpose::ReturnToHand, graveyard, rule)?;
    g.return_to_hand(&chosen);
    Ok(())
}

/// The creatures named Fang, Fearless l'Cie that `player` both owns and controls (CR 201.2, 108.3).
fn fangs(q: &dyn Query, player: PlayerId) -> Vec<ObjectId> {
    q.permanents_of(player).into_iter().filter(|&id| q.obj(id).owner == player && q.chars(id).is(CardType::Creature) && q.chars(id).name == FANG).collect()
}

/// "If you both own and control Vanille and a creature named Fang, Fearless l'Cie" (CR 603.4): asked as the ability
/// triggers and again as it resolves.
fn own_and_control_both(q: &dyn Query, asker: &Asker) -> bool {
    let you = asker.controller;
    triggers::this_is_on_the_battlefield(q, asker) && q.obj(asker.source).owner == you && q.controller(asker.source) == you && !fangs(q, you).is_empty()
}

const MELD: Question = Question::new("Meld", "pay {cost} to exile {subject} and Fang and meld them into Ragnarok, Divine Deliverance?", "", "Pay {cost}", "Don't pay");

/// "You may pay {3}{B}{G}. If you do, exile them, then meld them into Ragnarok, Divine Deliverance." (CR 701.42a, 712.4a,
/// 608.2c): with more than one Fang, the one exiled is chosen (the rulings); with one, it is that one. The two are
/// exiled together (CR 101.4) and melded as two cards in exile; a token or a card that left exile some other way is no
/// pair and stays where it is (CR 701.42c). Paying is what "if you do" asks, so with Fang gone by then Vanille is still
/// exiled (CR 609.3).
fn may_meld(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if !g.may_pay(you, item.object, item.object, cost("{3}{B}{G}"), &MELD)? {
        return Ok(());
    }
    let named = fangs(g, you);
    let chosen = if named.len() == 1 { named } else { choose::select(g, you, item.object, Purpose::Exile, named, select::CARD)? };
    // A Fang that left while the cost was paid (sacrificed for mana) leaves Vanille alone to exile: "exile them" still
    // exiles what it can, and only the meld fails (CR 608.2c, 609.3, 701.42c).
    let them: Vec<ObjectId> = std::iter::once(item.object).chain(chosen.first().copied()).collect();
    let exiled = g.exile_all(&them);
    g.end_sentence();
    let [vanille, fang] = exiled[..] else { return Ok(()) };
    g.meld(vanille, fang, you)?;
    Ok(())
}

/// "Target nonlegendary permanent card from your graveyard" (CR 115.1, 110.4a, 205.4a).
fn nonlegendary_permanent_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent_card_in_your_graveyard(q, asker, t) && !q.chars(id).supertypes.legendary)
}

/// "Destroy target permanent and return target nonlegendary permanent card from your graveyard to the battlefield." (CR
/// 701.8, 608.2b-c): in the order written, each target still legal as it resolves doing its part, an illegal one only its
/// own (CR 608.2b), the card returning under the ability's controller's control.
fn destroy_and_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    if let Some(card) = targets::object(g, item, 1) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

/// The meld card of Vanille, Cheerful l'Cie and Fang, Fearless l'Cie (CR 712.4a): a combined back face, which has no
/// characteristics outside the melded permanent (CR 712.4b). Reach, vigilance, menace, trample and haste are core keywords.
pub static RAGNAROK: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_DIES, destroy_and_return, "When Ragnarok dies, destroy target permanent and return target nonlegendary permanent card from your graveyard to the battlefield.")
        .targets(&[targets::PERMANENT, target(nonlegendary_permanent_card_in_your_graveyard)])],
    ..oracle::RAGNAROK_DIVINE_DELIVERANCE
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, mill_then_return, "When Vanille enters, mill two cards, then return a permanent card from your graveyard to your hand."),
        triggered(triggers::YOUR_FIRST_MAIN_PHASE, may_meld, "At the beginning of your first main phase, if you both own and control Vanille and a creature named Fang, Fearless l'Cie, you may pay {3}{B}{G}. If you do, exile them, then meld them into Ragnarok, Divine Deliverance.")
            .condition(own_and_control_both),
    ],
    meld: Some(Meld { with: FANG, into: &RAGNAROK }),
    ..oracle::VANILLE_CHEERFUL_LCIE
};
