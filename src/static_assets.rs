use eframe::epi;

use crate::utils::RawImage;

#[derive(Default)]
pub struct StaticAssets {
    pub background: RawImage,
    pub pskills: [RawImage; 4],
    pub xp: RawImage,
    pub mana: RawImage,
    pub luck: [RawImage; 7],
    pub morale: [RawImage; 7],
    pub flag: RawImage,
}

impl StaticAssets {
    pub fn init(&mut self, frame: &mut epi::Frame<'_>) {
        self.background
            .load_bytes(include_bytes!("../resources/heroscr4.png"), frame);
        self.pskills[0].load_bytes(include_bytes!("../resources/pskill_attack.png"), frame);
        self.pskills[1].load_bytes(include_bytes!("../resources/pskill_defence.png"), frame);
        self.pskills[2].load_bytes(include_bytes!("../resources/pskill_mpower.png"), frame);
        self.pskills[3].load_bytes(include_bytes!("../resources/pskill_knowledge.png"), frame);
        self.xp
            .load_bytes(include_bytes!("../resources/pskill_xp.png"), frame);
        self.mana
            .load_bytes(include_bytes!("../resources/pskill_mana.png"), frame);
        self.flag
            .load_bytes(include_bytes!("../resources/crest.png"), frame);

        self.luck[0].load_bytes(include_bytes!("../resources/luck/00_00.png"), frame);
        self.luck[1].load_bytes(include_bytes!("../resources/luck/00_01.png"), frame);
        self.luck[2].load_bytes(include_bytes!("../resources/luck/00_02.png"), frame);
        self.luck[3].load_bytes(include_bytes!("../resources/luck/00_03.png"), frame);
        self.luck[4].load_bytes(include_bytes!("../resources/luck/00_04.png"), frame);
        self.luck[5].load_bytes(include_bytes!("../resources/luck/00_05.png"), frame);
        self.luck[6].load_bytes(include_bytes!("../resources/luck/00_06.png"), frame);

        self.morale[0].load_bytes(include_bytes!("../resources/morale/00_00.png"), frame);
        self.morale[1].load_bytes(include_bytes!("../resources/morale/00_01.png"), frame);
        self.morale[2].load_bytes(include_bytes!("../resources/morale/00_02.png"), frame);
        self.morale[3].load_bytes(include_bytes!("../resources/morale/00_03.png"), frame);
        self.morale[4].load_bytes(include_bytes!("../resources/morale/00_04.png"), frame);
        self.morale[5].load_bytes(include_bytes!("../resources/morale/00_05.png"), frame);
        self.morale[6].load_bytes(include_bytes!("../resources/morale/00_06.png"), frame);
    }
}
