use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This artifact enters with X charge counters on it. {T}: Choose a color. Add one mana of that color for each charge counter on this artifact." (CR 614.1c, 122.6).
fn choose_color_add_one_mana_enters(
    _: &mut dyn Game,
    entering: &mut Entering,
) -> Result<(), Suspend> {
    let more = i64::from(entering.x).min(i32::MAX.into()) as i32;
    entering.counters[CounterKind::Charge as usize] = entering
        .counters[CounterKind::Charge as usize]
        .saturating_add(more);
    Ok(())
}

/// "{T}: Choose a color. Add one mana of that color for each charge counter on this artifact." (CR 605.1a, 106.3).
fn choose_color_add_one_mana_mana(
    g: &mut dyn Game,
    asker: &Asker,
    chosen: Option<Color>,
) -> Result<(), Suspend> {
    let Some(chosen_1) = chosen else { return Ok(()) };
    let mana_3_0 = {
        let q: &dyn Query = g;
        Some(asker.source)
            .into_iter()
            .map(|o: ObjectId| i64::from(q.state().obj(o).counters(CounterKind::Charge)))
            .sum::<i64>() as u32
    };
    g.add_mana(asker.controller, chosen_1.mana(), mana_3_0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_any_color(1).by(choose_color_add_one_mana_mana),
            "{T}: Choose a color. Add one mana of that color for each charge counter on this artifact.",
        ),
    ],
    enters: Some(choose_color_add_one_mana_enters),
    ..oracle::ASTRAL_CORNUCOPIA
};
