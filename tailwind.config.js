/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			screens: {
				'3xl': '1792px'
			}
		}
		// animatedSettings: {
		//     classes: ["fadeIn"],
		// },
		// plugins: [require("tailwindcss-animatecss")],
	}
};
