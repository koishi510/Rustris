pub enum Sfx {
    Move,
    Rotate,
    HardDrop,
    Hold,
    Lock,
    LineClear(u32),
    LevelUp,
    GameOver,
    MenuMove,
    MenuSelect,
    Pause,
    Resume,
    TSpinMini,
    TSpin,
    TSpinClear(u32),
    AllClear,
    Combo(u32),
    BackToBack,
    Clear,
    MenuBack,
    GarbageReceived,
    VersusWin,
    VersusLose,
}

impl Sfx {
    pub(super) fn notes(&self) -> Vec<(f32, u32)> {
        match self {
            Sfx::Move => vec![(440.0, 20)],
            Sfx::Rotate => vec![(523.0, 25), (659.0, 25)],
            Sfx::HardDrop => vec![(220.0, 25), (110.0, 40)],
            Sfx::Hold => vec![(587.0, 30), (784.0, 30)],
            Sfx::Lock => vec![(247.0, 25), (220.0, 35)],
            Sfx::LineClear(n) => match n {
                1 => vec![(523.0, 50), (659.0, 60)],
                2 => vec![(523.0, 40), (659.0, 40), (784.0, 55)],
                3 => vec![(523.0, 35), (659.0, 35), (784.0, 35), (1047.0, 60)],
                _ => vec![(784.0, 40), (988.0, 40), (1175.0, 40), (1568.0, 100)],
            },
            Sfx::LevelUp => vec![
                (523.0, 50),
                (659.0, 50),
                (784.0, 50),
                (1047.0, 80),
                (1319.0, 100),
            ],
            Sfx::GameOver => vec![(440.0, 150), (370.0, 150), (311.0, 150), (247.0, 300)],
            Sfx::MenuMove => vec![(660.0, 15)],
            Sfx::MenuSelect => vec![(523.0, 40), (784.0, 40), (1047.0, 60)],
            Sfx::Pause => vec![(400.0, 60), (300.0, 80)],
            Sfx::Resume => vec![(300.0, 60), (400.0, 80)],
            Sfx::TSpinMini => vec![(659.0, 30), (784.0, 30), (659.0, 40)],
            Sfx::TSpin => vec![(523.0, 35), (784.0, 35), (1047.0, 50)],
            Sfx::TSpinClear(n) => match n {
                1 => vec![(659.0, 40), (784.0, 40), (1047.0, 60)],
                2 => vec![(659.0, 35), (784.0, 35), (1047.0, 35), (1319.0, 70)],
                _ => vec![(784.0, 35), (1047.0, 35), (1319.0, 35), (1568.0, 80)],
            },
            Sfx::AllClear => vec![
                (1047.0, 50),
                (1319.0, 50),
                (1568.0, 50),
                (2093.0, 50),
                (1568.0, 40),
                (2093.0, 80),
            ],
            Sfx::Combo(n) => {
                let base = 523.0 + *n as f32 * 50.0;
                vec![(base, 25), (base * 1.25, 35)]
            }
            Sfx::BackToBack => vec![(880.0, 30), (1047.0, 30), (1319.0, 50)],
            Sfx::Clear => vec![
                (784.0, 80), (988.0, 80), (1175.0, 80),
                (1568.0, 100), (1175.0, 60), (1568.0, 150),
            ],
            Sfx::MenuBack => vec![(523.0, 30), (392.0, 50)],
            Sfx::GarbageReceived => vec![(200.0, 40), (150.0, 60)],
            Sfx::VersusWin => vec![
                (784.0, 80), (988.0, 80), (1175.0, 80),
                (1568.0, 100), (1175.0, 60), (1568.0, 200),
            ],
            Sfx::VersusLose => vec![(440.0, 150), (370.0, 150), (311.0, 200), (247.0, 350)],
        }
    }
}
