import pytest
import math

# プロジェクト名(Cargo.tomlのlib name)に合わせてインポート
import inverse_observation_induced_probability_field_interference


def test_constructive_interference():
    """
    真のカメラ位置において、確率密度（干渉）が最大化することを確認する。
    """
    # 1. Initialize Simulation
    wave_number = 10.0
    sim = inverse_observation_induced_probability_field_interference.PyQuantumSlam(
        wave_number
    )

    # 2. Add Landmarks (e.g., Triangle formation)
    sim.add_landmark(0.0, 10.0)
    sim.add_landmark(-10.0, -10.0)
    sim.add_landmark(10.0, -10.0)

    # 3. Simulate Observation
    # True Camera is at (0,0)
    true_x, true_y = 0.0, 0.0
    sim.update_observation(true_x, true_y)

    # 4. Check Probability Field
    prob_at_center = sim.get_probability(0.0, 0.0)
    prob_at_off = sim.get_probability(5.0, 5.0)  # Wrong location

    print(f"Prob(Center): {prob_at_center}")
    print(f"Prob(Offset): {prob_at_off}")

    # 干渉により、中心（真の値）の方が確率が高いはず
    assert prob_at_center > prob_at_off, "Constructive interference failed!"


def test_interference_resolution():
    """
    波数(k)を上げると、ピークが鋭くなる（不確定性が減る）ことを確認
    """
    # Low K vs High K
    sim_low_k = (
        inverse_observation_induced_probability_field_interference.PyQuantumSlam(1.0)
    )
    sim_high_k = (
        inverse_observation_induced_probability_field_interference.PyQuantumSlam(50.0)
    )

    # 【修正ポイント】
    # 干渉（Interference）を発生させるためには、最低でも2つの波源（ランドマーク）が必要です。
    # 1つだけだと、単なる距離減衰（エンベロープ）しか観測されず、波数の影響が振幅に出ません。
    # 対向する位置に2つ配置することで、中心からずれた時にお互いの位相が打ち消し合う効果を確認します。

    landmarks = [(10.0, 0.0), (-10.0, 0.0)]

    for x, y in landmarks:
        sim_low_k.add_landmark(x, y)
        sim_high_k.add_landmark(x, y)

    # 真のカメラ位置(0,0)で観測を更新
    sim_low_k.update_observation(0.0, 0.0)
    sim_high_k.update_observation(0.0, 0.0)

    # 少しだけズレた場所
    offset = 0.1

    # Low K: ズレても位相差が小さいため、強め合いが続き、確率はあまり下がらない (Broad peak)
    prob_low_center = sim_low_k.get_probability(0.0, 0.0)
    prob_low_offset = sim_low_k.get_probability(offset, 0.0)
    decay_low = prob_low_center - prob_low_offset

    # High K: ズレると位相差が大きくなり、破壊的干渉が起きて確率は急激に下がる (Sharp peak)
    prob_high_center = sim_high_k.get_probability(0.0, 0.0)
    prob_high_offset = sim_high_k.get_probability(offset, 0.0)
    decay_high = prob_high_center - prob_high_offset

    print(
        f"\n[Low K] Center: {prob_low_center:.4f}, Offset: {prob_low_offset:.4f}, Decay: {decay_low:.4f}"
    )
    print(
        f"[High K] Center: {prob_high_center:.4f}, Offset: {prob_high_offset:.4f}, Decay: {decay_high:.4f}"
    )

    # High Kの方が、少しのズレで大きく値が下がる（減衰量が大きい）はず
    assert decay_high > decay_low, (
        f"Quantum precision check failed! High K should decay faster. (L:{decay_low} vs H:{decay_high})"
    )


if __name__ == "__main__":
    test_constructive_interference()
    test_interference_resolution()
    print("All Quantum Tests Passed.")
