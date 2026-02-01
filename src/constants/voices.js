export const LANGUAGES = [
    { code: 'ja', name: 'Japanese', flag: '🇯🇵' },
    { code: 'vi', name: 'Tiếng Việt', flag: '🇻🇳' },
    { code: 'en', name: 'English', flag: '🇺🇸' },
    { code: 'ko', name: 'Korean', flag: '🇰🇷' },
];

export const VOICES_DATA = {
    vi: {
        female: [
            { id: 'vi-vn-f-default', voice: 'vi-VN-HoaiMyNeural', name: 'Hoài My', desc: 'Giọng nữ chuẩn', icon: '�', pitch: 0, rate: 0, bass: 0, treble: 0 },
        ],
        male: [
            { id: 'vi-vn-m-default', voice: 'vi-VN-NamMinhNeural', name: 'Nam Minh', desc: 'Giọng nam chuẩn', icon: '�', pitch: 0, rate: 0, bass: 0, treble: 0 },
        ]
    },
    en: {
        female: [
            { id: 'en-us-f-default', voice: 'en-US-AvaNeural', name: 'Ava', desc: 'US Female', icon: '�', pitch: 0, rate: 0 },
        ],
        male: [
            { id: 'en-us-m-default', voice: 'en-US-AndrewNeural', name: 'Andrew', desc: 'US Male', icon: '�', pitch: 0, rate: 0 },
        ]
    },
    ja: {
        female: [
            { id: 'ja-jp-f-default', voice: 'ja-JP-NanamiNeural', name: 'Nanami', desc: 'Japanese Female', icon: '�‍🏫', pitch: 0, rate: 0 },
        ],
        male: [
            { id: 'ja-jp-m-default', voice: 'ja-JP-KeitaNeural', name: 'Keita', desc: 'Japanese Male', icon: '�', pitch: 0, rate: 0 },
        ]
    },
    ko: {
        female: [
            { id: 'ko-kr-f-default', voice: 'ko-KR-SunHiNeural', name: 'Sun-Hi', desc: 'Korean Female', icon: '👩', pitch: 0, rate: 0 },
        ],
        male: [
            { id: 'ko-kr-m-default', voice: 'ko-KR-InJoonNeural', name: 'In-Joon', desc: 'Korean Male', icon: '�', pitch: 0, rate: 0 },
        ]
    }
};

export const ALL_VOICES = [
    ...Object.values(VOICES_DATA).flatMap(v => v.female),
    ...Object.values(VOICES_DATA).flatMap(v => v.male)
];
