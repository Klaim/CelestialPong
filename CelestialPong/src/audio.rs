use macroquad::audio::*;

const DEFAULT_BGM_VOLUME : f32 = 0.8;

pub async fn start_bgm()
{
    let bgm = load_sound_from_bytes(include_bytes!("../audio/BGM-celestial_poing-by_Klaim.mp3")).await.unwrap();
    play_sound(&bgm, PlaySoundParams{
        looped: true,
        volume: DEFAULT_BGM_VOLUME,
    })
}



