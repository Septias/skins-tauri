<template lang="pug">
div.flex.flex-col.justify-between.border.border-rose-500.rounded-xl.p-2
  p.text-red.font-bold(v-if="chest.error") {{ chest.error }}
  div.flex.justify-between.items-center {{ chest.name }}
  div.flex.justify-center
    img(:src="'https://community.akamai.steamstatic.com/economy/image/' + chest.icon_url")
  p.text-red.font-bold(v-if="chest.values") {{ chest.values }}

</template>

<script lang="ts" setup>
import type { PropType } from 'vue';
import { MarketItemOnCrack } from '../pages/all_items.vue';

defineProps({
  chest: Object as PropType<MarketItemOnCrack>
})
</script>
