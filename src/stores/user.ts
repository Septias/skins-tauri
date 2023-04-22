import { acceptHMRUpdate, defineStore } from 'pinia'

export const useUsersStore = defineStore('config', () => {
  const username = useStorage('userid', "")
  return {
    username,
  }
})

if (import.meta.hot)
  import.meta.hot.accept(acceptHMRUpdate(useUsersStore, import.meta.hot))
