<script lang="ts">
	import { listen } from "@tauri-apps/api/event";
	let events = [];
	// default example
	listen("quickening", (event) => {
	//listen("yourEventName", (event) => {
		events.push(JSON.stringify(event));
		events = events; 
		/* 	The above is due to the following issue
			https://github.com/sveltejs/svelte/issues/2362 
		   	Svelte doesn't treat .push as an update trigger
		   	but assignment does trigger a re-render cycle
		*/
		console.log(event);
	});
</script>
<style>

</style>
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
