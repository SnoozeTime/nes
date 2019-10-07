CPU_FREQ = 1789773.0

def midi_note_to_nes_timer(freq):
    timer = (CPU_FREQ/(16 * freq)) - 1
    return round(timer)


