export const LANGUAGES = [
    { code: 'vi', name: 'Tiếng Việt', flag: '🇻🇳' },
    { code: 'en', name: 'English', flag: '🇺🇸' },
    { code: 'ja', name: 'Japanese', flag: '🇯🇵' },
    { code: 'ko', name: 'Korean', flag: '🇰🇷' },
];

export const VOICES_DATA = {
    vi: {
        female: [
            { id: 'vi-vn-f-5', voice: 'vi-VN-HoaiMyNeural', name: 'Bé Na (5 tuổi)', desc: 'Ngây thơ, cao vút', icon: '👧', pitch: 95, rate: 15, bass: -20, treble: 20 },
            { id: 'vi-vn-f-6', voice: 'vi-VN-HoaiMyNeural', name: 'Bé Cốm (6 tuổi)', desc: 'Líu lo, dễ thương', icon: '🍭', pitch: 85, rate: 10, bass: -15, treble: 18 },
            { id: 'vi-vn-f-8', voice: 'vi-VN-HoaiMyNeural', name: 'Bé Bống (8 tuổi)', desc: 'Tinh nghịch, lanh lợi', icon: '👧', pitch: 65, rate: 10, bass: -10, treble: 15 },
            { id: 'vi-vn-f-8b', voice: 'vi-VN-HoaiMyNeural', name: 'Bé Thỏ (8 tuổi)', desc: 'Nhút nhát, nhẹ nhàng', icon: '🐰', pitch: 75, rate: 0, bass: -15, treble: 20 },
            { id: 'vi-vn-f-9', voice: 'vi-VN-HoaiMyNeural', name: 'Bé Mây (9 tuổi)', desc: 'Trong veo, nắng sớm', icon: '☁️', pitch: 55, rate: 5, bass: -5, treble: 10 },
            { id: 'vi-vn-f-12', voice: 'vi-VN-HoaiMyNeural', name: 'Chị Chanh (12 tuổi)', desc: 'Trong trẻo, hồn nhiên', icon: '🎒', pitch: 40, rate: 5, bass: -5, treble: 8 },
            { id: 'vi-vn-f-15', voice: 'vi-VN-HoaiMyNeural', name: 'Hạ Vy (15 tuổi)', desc: 'Mộng mơ, nữ sinh', icon: '🎀', pitch: 25, rate: 2, bass: -2, treble: 5 },
            { id: 'vi-vn-f-18', voice: 'vi-VN-HoaiMyNeural', name: 'Tú Anh (18 tuổi)', desc: 'Năng động, trẻ trung', icon: '🎓', pitch: 10, rate: 0, bass: 0, treble: 5 },
            { id: 'vi-vn-f-22', voice: 'vi-VN-HoaiMyNeural', name: 'Ngọc Diệp (22 tuổi)', desc: 'Thanh lịch, chuẩn mực', icon: '👩', pitch: 0, rate: 0, bass: 0, treble: 2 },
            { id: 'vi-vn-f-25', voice: 'vi-VN-HoaiMyNeural', name: 'Mai Phương (25 tuổi)', desc: 'Công sở, tự tin', icon: '👔', pitch: -5, rate: 0, bass: 2, treble: 0 },
            { id: 'vi-vn-f-30', voice: 'vi-VN-HoaiMyNeural', name: 'Thu Thủy (30 tuổi)', desc: 'Quyến rũ, trưởng thành', icon: '💃', pitch: -15, rate: -2, bass: 8, treble: -2 },
            { id: 'vi-vn-f-35', voice: 'vi-VN-HoaiMyNeural', name: 'Hồng Hạnh (35 tuổi)', desc: 'Sâu sắc, truyền cảm', icon: '🎙️', pitch: -25, rate: -5, bass: 12, treble: -5 },
            { id: 'vi-vn-f-38', voice: 'vi-VN-HoaiMyNeural', name: 'Kim Chi (38 tuổi)', desc: 'Uy quyền, quyết đoán', icon: '👑', pitch: -30, rate: 0, bass: 15, treble: -10 },
            { id: 'vi-vn-f-40', voice: 'vi-VN-HoaiMyNeural', name: 'Thanh Hà (40 tuổi)', desc: 'Hiền hậu, giọng mẹ', icon: '❤️', pitch: -20, rate: -8, bass: 10, treble: 0 },
            { id: 'vi-vn-f-adult', voice: 'vi-VN-HoaiMyNeural', name: 'Cô Lan (Adult)', desc: 'Giọng đọc chuẩn', icon: '👩‍🏫', pitch: 0, rate: 0, bass: 0, treble: 0 },
        ],
        male: [
            { id: 'vi-vn-m-5', voice: 'vi-VN-NamMinhNeural', name: 'Cu Tí (5 tuổi)', desc: 'Ngọng nghịu, đáng yêu', icon: '🍼', pitch: 85, rate: 12, bass: -15, treble: 15 },
            { id: 'vi-vn-m-7', voice: 'vi-VN-NamMinhNeural', name: 'Bé Bi (7 tuổi)', desc: 'Hiếu động, nghịch ngợm', icon: '👦', pitch: 65, rate: 8, bass: -10, treble: 12 },
            { id: 'vi-vn-m-8', voice: 'vi-VN-NamMinhNeural', name: 'Bé Gấu (8 tuổi)', desc: 'Hơi trầm, mũm mĩm', icon: '🧸', pitch: 45, rate: -5, bass: 5, treble: 5 },
            { id: 'vi-vn-m-10', voice: 'vi-VN-NamMinhNeural', name: 'Bé Sóc (10 tuổi)', desc: 'Nhanh nhẹn, thông minh', icon: '🐿️', pitch: 35, rate: 10, bass: -5, treble: 8 },
            { id: 'vi-vn-m-14', voice: 'vi-VN-NamMinhNeural', name: 'Hoàng Long (14 tuổi)', desc: 'Đang vỡ giọng', icon: '⚽', pitch: 20, rate: 2, bass: 2, treble: 0 },
            { id: 'vi-vn-m-17', voice: 'vi-VN-NamMinhNeural', name: 'Gia Bảo (17 tuổi)', desc: 'Thư sinh, nhẹ nhàng', icon: '📚', pitch: 5, rate: 0, bass: 5, treble: 0 },
            { id: 'vi-vn-m-20', voice: 'vi-VN-NamMinhNeural', name: 'Minh Quân (20 tuổi)', desc: 'Trẻ trung, nhiệt huyết', icon: '🔥', pitch: 0, rate: 5, bass: 0, treble: 2 },
            { id: 'vi-vn-m-24', voice: 'vi-VN-NamMinhNeural', name: 'Quốc Khánh (24 tuổi)', desc: 'Trầm ổn, đĩnh đạc', icon: '💼', pitch: -10, rate: 0, bass: 8, treble: -2 },
            { id: 'vi-vn-m-26', voice: 'vi-VN-NamMinhNeural', name: 'Nam Minh (26 tuổi)', desc: 'Mạnh mẽ, chuẩn mực', icon: '👨', pitch: 0, rate: 0, bass: 0, treble: 0 },
            { id: 'vi-vn-m-28', voice: 'vi-VN-NamMinhNeural', name: 'Anh Đức (28 tuổi)', desc: 'Ấm áp, tin cậy', icon: '🤝', pitch: -15, rate: -2, bass: 10, treble: 0 },
            { id: 'vi-vn-m-32', voice: 'vi-VN-NamMinhNeural', name: 'Quang Huy (32 tuổi)', desc: 'Trưởng thành, lịch lãm', icon: '🕺', pitch: -25, rate: 0, bass: 12, treble: -5 },
            { id: 'vi-vn-m-35', voice: 'vi-VN-NamMinhNeural', name: 'Hùng Dũng (35 tuổi)', desc: 'Giọng dày, uy lực', icon: '💪', pitch: -35, rate: -5, bass: 20, treble: -8 },
            { id: 'vi-vn-m-38', voice: 'vi-VN-NamMinhNeural', name: 'Bác Thành (38 tuổi)', desc: 'Kinh nghiệm, sâu lắng', icon: '👴', pitch: -40, rate: -10, bass: 15, treble: -5 },
            { id: 'vi-vn-m-40', voice: 'vi-VN-NamMinhNeural', name: 'Chú Bình (40 tuổi)', desc: 'Nghiêm nghị, ấm áp', icon: '🏠', pitch: -30, rate: -5, bass: 10, treble: 2 },
            { id: 'vi-vn-m-adult', voice: 'vi-VN-NamMinhNeural', name: 'Chú Hùng (Adult)', desc: 'Giọng kể chuyện', icon: '📖', pitch: -10, rate: -15, bass: 5, treble: 0 },
        ]
    },
    en: {
        female: [
            { id: 'en-us-female-1', voice: 'en-US-AvaNeural', name: 'Ava', desc: 'Cheerful', icon: '👧', pitch: 0, rate: 0 },
            { id: 'en-us-female-2', voice: 'en-US-EmmaNeural', name: 'Emma', desc: 'Sweet', icon: '👩', pitch: 0, rate: 0 },
        ],
        male: [
            { id: 'en-us-male-1', voice: 'en-US-AndrewNeural', name: 'Andrew', desc: 'Professional', icon: '👨', pitch: 0, rate: 0 },
            { id: 'en-us-male-2', voice: 'en-US-BrianNeural', name: 'Brian', desc: 'Deep voice', icon: '🧔', pitch: 0, rate: 0 },
        ]
    },
    ja: {
        female: [
            { id: 'ja-jp-female-1', voice: 'ja-JP-NanamiNeural', name: 'Nanami (Chuẩn)', desc: 'Giáo viên', icon: '👩‍🏫', pitch: 0, rate: 0 },
        ],
        male: [
            { id: 'ja-jp-male-1', voice: 'ja-JP-KeitaNeural', name: 'Keita', desc: 'Thanh niên', icon: '👦', pitch: 0, rate: 0 },
        ]
    }
};

export const ALL_VOICES = [
    ...Object.values(VOICES_DATA).flatMap(v => v.female),
    ...Object.values(VOICES_DATA).flatMap(v => v.male)
];
