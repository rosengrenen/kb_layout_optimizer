import React from "react";

import s from "./index.page.module.scss";

interface Button {
	hand: Hand;
	finger: Finger;
	position: { x: number; y: number };
	matrix_position: { x: number; y: number };
}

enum Hand {
	LEFT = "Left",
	RIGHT = "Right",
}

enum Finger {
	THUMB = "Thumb",
	POINTER = "Pointer",
	MIDDLE = "Middle",
	RING = "Ring",
	PINKY = "Pinky",
}

const Home = () => {
	return (
		<div>
			<KeyCap />
		</div>
	);
};

const KeyCap = () => {
	return (
		<div className={s["keycap"]}>
			<div className={s["keycap-top"]}>A</div>
		</div>
	);
};

export default Home;
