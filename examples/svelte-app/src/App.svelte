<script lang="ts">
	import { listen } from "@tauri-apps/api/event";
	let events = [];
	listen("opener", (event) => {
		events.push(JSON.stringify(event));
	});
</script>

<div>
	<h1>Events</h1>
	<div>
		<ul>
			{#each events as event}
				<li>{event}</li>
			{/each}
		</ul>
	</div>
</div>
