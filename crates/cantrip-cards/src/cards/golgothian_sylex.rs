use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{1}, {T}: Each nontoken permanent with a name originally printed in the Antiquities expansion is sacrificed by its controller." (CR 608.2, 701.21a).
fn nontoken_permanent_name_originally_printed(
    g: &mut dyn Game,
    _: &StackItem,
) -> Result<(), Suspend> {
    let objects_0 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                !(q.state().obj(id).token)
                    && matches!(
                        q.chars(id).name, "Amulet of Kroog" | "Argivian Archaeologist" |
                        "Argivian Blacksmith" | "Argothian Pixies" | "Argothian Treefolk"
                        | "Armageddon Clock" | "Artifact Blast" | "Artifact Possession" |
                        "Artifact Ward" | "Ashnod's Altar" | "Ashnod's Battle Gear" |
                        "Ashnod's Transmogrant" | "Atog" | "Battering Ram" |
                        "Bronze Tablet" | "Candelabra of Tawnos" |
                        "Circle of Protection: Artifacts" | "Citanul Druid" |
                        "Clay Statue" | "Clockwork Avian" | "Colossus of Sardia" |
                        "Coral Helm" | "Crumble" | "Cursed Rack" | "Damping Field" |
                        "Detonate" | "Drafna's Restoration" | "Dragon Engine" |
                        "Dwarven Weaponsmith" | "Energy Flux" | "Feldon's Cane" |
                        "Gaea's Avenger" | "Gate to Phyrexia" | "Goblin Artisans" |
                        "Golgothian Sylex" | "Grapeshot Catapult" | "Haunting Wind" |
                        "Hurkyl's Recall" | "Ivory Tower" | "Jalum Tome" |
                        "Martyrs of Korlis" | "Mightstone" | "Millstone" |
                        "Mishra's Factory" | "Mishra's War Machine" | "Mishra's Workshop"
                        | "Obelisk of Undoing" | "Onulet" | "Orcish Mechanics" |
                        "Ornithopter" | "Phyrexian Gremlins" | "Power Artifact" |
                        "Powerleech" | "Priest of Yawgmoth" | "Primal Clay" | "Rakalite"
                        | "Reconstruction" | "Reverse Polarity" | "Rocket Launcher" |
                        "Sage of Lat-Nam" | "Shapeshifter" | "Shatterstorm" |
                        "Staff of Zegon" | "Strip Mine" | "Su-Chi" | "Tablet of Epityr" |
                        "Tawnos's Coffin" | "Tawnos's Wand" | "Tawnos's Weaponry" |
                        "Tetravus" | "The Rack" | "Titania's Song" | "Transmute Artifact"
                        | "Triskelion" | "Urza's Avenger" | "Urza's Chalice" |
                        "Urza's Mine" | "Urza's Miter" | "Urza's Power Plant" |
                        "Urza's Tower" | "Wall of Spears" | "Weakstone" |
                        "Xenic Poltergeist" | "Yawgmoth Demon" | "Yotian Soldier"
                    )
            })
            .collect::<Vec<ObjectId>>()
    };
    let picks_0: Vec<(PlayerId, ObjectId)> = objects_0
        .into_iter()
        .map(|object| (g.controller(object), object))
        .collect();
    g.sacrifice_each(&picks_0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}").and(TAP),
            nontoken_permanent_name_originally_printed,
            "{1}, {T}: Each nontoken permanent with a name originally printed in the Antiquities expansion is sacrificed by its controller.",
        ),
    ],
    ..oracle::GOLGOTHIAN_SYLEX
};
