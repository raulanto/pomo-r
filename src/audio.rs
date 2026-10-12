use std::f32::consts::PI;
use std::time::Duration;

use crate::domain::Phase;

/// Frecuencia de muestreo estándar para síntesis de audio.
const SAMPLE_RATE: u32 = 44100;

/// Genera un búfer estéreo con el sonido ambiental correspondiente a la transición de fase.
///
/// - Transición a Descanso (tras Trabajo): cuenco tibetano / campana zen cálida a 432 Hz,
///   con subarmónicos, batimiento binaural estéreo y decaimiento orgánico largo (3s).
/// - Transición a Trabajo (tras Descanso): carillón cristalino de dos notas armónicas
///   ascendentes con apertura espacial y resonancia suave (2.4s).
pub fn generate_sound(next_phase: Phase) -> Vec<f32> {
    match next_phase {
        Phase::ShortBreak | Phase::LongBreak => generate_zen_bowl(),
        Phase::Work => generate_crystal_chime(),
    }
}

/// Sonido Zen / Cuenco Tibetano para entrar en descanso.
/// Profundo, envolvente, con resonancia metálica cálida y respiración estéreo.
fn generate_zen_bowl() -> Vec<f32> {
    let duration_secs = 2.8f32;
    let total_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let mut buffer = Vec::with_capacity(total_samples * 2);

    let f0 = 432.0f32; // Fundamental cálida
    let sub = 216.0f32; // Calidez / cuerpo inferior
    let h1 = 864.0f32; // Primera octava
    let partial1 = 1170.0f32; // Modo armónico de campana
    let partial2 = 1536.0f32; // Brillo sutil

    for i in 0..total_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Ataque suave (fade-in) para evitar cualquier "clic" o chasquido
        let attack = if t < 0.04 {
            (t / 0.04 * (PI / 2.0)).sin()
        } else {
            1.0
        };

        // Decaimiento exponencial natural (los agudos caen antes que los graves)
        let decay_fund = (-t / 1.15).exp();
        let decay_sub = (-t / 1.25).exp();
        let decay_h1 = (-t / 0.85).exp();
        let decay_p1 = (-t / 0.60).exp();
        let decay_p2 = (-t / 0.45).exp();

        // Micro-desafinación (binaural beat) para dar espacialidad y profundidad envolvente
        let beat_l = (2.0 * PI * 0.35 * t).sin() * 0.15;
        let beat_r = (2.0 * PI * 0.35 * t + PI * 0.5).sin() * 0.15;

        // Canal izquierdo
        let left_signal = (2.0 * PI * (f0 - 0.4) * t).sin() * (0.45 + beat_l) * decay_fund
            + (2.0 * PI * sub * t).sin() * 0.28 * decay_sub
            + (2.0 * PI * (h1 + 0.3) * t).sin() * 0.15 * decay_h1
            + (2.0 * PI * partial1 * t).sin() * 0.08 * decay_p1
            + (2.0 * PI * partial2 * t).sin() * 0.04 * decay_p2;

        // Canal derecho
        let right_signal = (2.0 * PI * (f0 + 0.4) * t).sin() * (0.45 + beat_r) * decay_fund
            + (2.0 * PI * sub * t).sin() * 0.28 * decay_sub
            + (2.0 * PI * (h1 - 0.3) * t).sin() * 0.15 * decay_h1
            + (2.0 * PI * (partial1 * 1.002) * t).sin() * 0.08 * decay_p1
            + (2.0 * PI * (partial2 * 0.998) * t).sin() * 0.04 * decay_p2;

        let left = (left_signal * attack * 0.42).clamp(-1.0, 1.0);
        let right = (right_signal * attack * 0.42).clamp(-1.0, 1.0);

        buffer.push(left);
        buffer.push(right);
    }

    buffer
}

/// Carillón cristalino de dos notas suaves ascendentes para reiniciar el trabajo.
/// Limpio, nítido y reconfortante.
fn generate_crystal_chime() -> Vec<f32> {
    let duration_secs = 2.4f32;
    let total_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let mut buffer = Vec::with_capacity(total_samples * 2);

    let note1_time = 0.0f32;
    let note2_time = 0.18f32;

    let f1 = 528.0f32; // Nota 1: Solfeggio 528 Hz (claridad)
    let f2 = 792.0f32; // Nota 2: Quinta perfecta (792 Hz)

    for i in 0..total_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Nota 1
        let (n1_l, n1_r) = if t >= note1_time {
            let dt = t - note1_time;
            let att = if dt < 0.025 {
                (dt / 0.025 * (PI / 2.0)).sin()
            } else {
                1.0
            };
            let decay = (-dt / 0.95).exp();
            let wave = (2.0 * PI * f1 * dt).sin() * 0.38
                + (2.0 * PI * (f1 * 2.0) * dt).sin() * 0.12 * (-dt / 0.6).exp();
            // Ligera inclinación hacia la izquierda
            (wave * att * decay * 0.85, wave * att * decay * 0.55)
        } else {
            (0.0, 0.0)
        };

        // Nota 2
        let (n2_l, n2_r) = if t >= note2_time {
            let dt = t - note2_time;
            let att = if dt < 0.025 {
                (dt / 0.025 * (PI / 2.0)).sin()
            } else {
                1.0
            };
            let decay = (-dt / 1.05).exp();
            let wave = (2.0 * PI * f2 * dt).sin() * 0.36
                + (2.0 * PI * (f2 * 2.0) * dt).sin() * 0.10 * (-dt / 0.6).exp();
            // Ligera inclinación hacia la derecha
            (wave * att * decay * 0.55, wave * att * decay * 0.85)
        } else {
            (0.0, 0.0)
        };

        // Eco ambiental sutil
        let echo_dt = t - 0.28;
        let (echo_l, echo_r) = if echo_dt > 0.0 {
            let decay = (-echo_dt / 0.7).exp();
            let w = (2.0 * PI * f1 * echo_dt).sin() * 0.06 * decay;
            (w * 0.5, w * 0.8)
        } else {
            (0.0, 0.0)
        };

        let left = ((n1_l + n2_l + echo_l) * 0.38).clamp(-1.0, 1.0);
        let right = ((n1_r + n2_r + echo_r) * 0.38).clamp(-1.0, 1.0);

        buffer.push(left);
        buffer.push(right);
    }

    buffer
}

/// Reproduce el sonido de cambio de fase de manera asíncrona en un hilo independiente.
/// Nunca bloquea el bucle de eventos ni hace panic si no hay dispositivo de audio.
pub fn play_phase_sound_async(next_phase: Phase) {
    std::thread::spawn(move || {
        play_phase_sound(next_phase);
    });
}

/// Reproduce el sonido correspondiente usando `rodio`.
/// Si el hardware de audio no está disponible o falla, simplemente no hace nada (no panic).
pub fn play_phase_sound(next_phase: Phase) {
    let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() else {
        return;
    };

    let Ok(sink) = rodio::Sink::try_new(&stream_handle) else {
        return;
    };

    let samples = generate_sound(next_phase);
    let source = rodio::buffer::SamplesBuffer::new(2, SAMPLE_RATE, samples);

    sink.append(source);
    sink.sleep_until_end();
    // Breve pausa para asegurar que el backend termine de vaciar los buffers
    std::thread::sleep(Duration::from_millis(50));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sound_lengths_and_finite() {
        for phase in [Phase::Work, Phase::ShortBreak, Phase::LongBreak] {
            let samples = generate_sound(phase);
            // Debe ser estéreo (número par de muestras)
            assert_eq!(samples.len() % 2, 0);
            assert!(samples.len() > 44100); // al menos 1 segundo
            for &sample in &samples {
                assert!(sample.is_finite());
                assert!((-1.0..=1.0).contains(&sample));
            }
        }
    }

    #[test]
    fn test_play_phase_sound_no_panic() {
        // En CI o máquinas sin audio, no debe hacer panic
        play_phase_sound(Phase::ShortBreak);
        play_phase_sound(Phase::Work);
    }
}
