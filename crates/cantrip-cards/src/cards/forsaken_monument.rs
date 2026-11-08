use cantrip_api::*;
use cantrip_mechanics::{effects, statics};
use cantrip_oracle as oracle;

/// "Colorless creatures you control get +2/+2." (CR 613.4c, layer 7c): every colorless creature its controller controls,
/// judged on what the earlier layers made the object.
fn colorless_creatures_you_control(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    so_far.is(CardType::Creature) && so_far.colors.is_empty() && q.controller(affected) == q.controller(source)
}

/// "Whenever you tap a permanent for {C}" (CR 106.12a, 605.1b): a permanent this ability's controller taps, whose mana
/// ability produced colorless mana, land or not - the permanent is the subject and its tapper is in `data[0]`.
fn you_tap_a_permanent_for_colorless(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match *event {
        Event::TappedForMana { permanent, player, produced }
            if player == q.controller(source) && produced.get(Mana::C) > 0 =>
        {
            Some(Fired { subject: Target::Object(permanent), data: [player.0 as u32, 0] })
        }
        _ => None,
    }
}

/// "add an additional {C}." (CR 605.4a): a triggered mana ability, so it is added as the mana ability that triggered it
/// resolves, by the player who tapped the permanent.
fn add_an_additional_colorless(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(PlayerId(item.data[0] as u8), Mana::C, 1);
    Ok(())
}

/// "Whenever you cast a colorless spell" (CR 603.2): judged on the spell as it is on the stack, not by its mana cost.
fn colorless_spell_you_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source) && q.chars(*spell).colors.is_empty() =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        colorless_creatures_you_control,
        statics::plus_pt::<2, 2>,
        "Colorless creatures you control get +2/+2.",
    )],
    abilities: &[
        triggered(
            when(EventMask::TAPPED, you_tap_a_permanent_for_colorless),
            add_an_additional_colorless,
            "Whenever you tap a permanent for {C}, add an additional {C}.",
        )
        .mana_ability(TriggeredMana::One(Mana::C)),
        triggered(
            when(EventMask::SPELL_CAST, colorless_spell_you_cast),
            |g, item| effects::gain_life(g, item, 2),
            "Whenever you cast a colorless spell, you gain 2 life.",
        ),
    ],
    ..oracle::FORSAKEN_MONUMENT
};
