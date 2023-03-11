<script lang="ts">
import { parse } from "rotext-renderer-vue";

export default {
  props: ["markup"],
  emits: ["updateCharacterCount", "updateParsingTime", "error"],
  setup(props, { emit }) {
    return () => {
      const charCount = [...new Intl.Segmenter().segment(props.markup)].length;
      emit("updateCharacterCount", charCount);
      try {
        const parsingStart = performance.now();
        const result = parse(props.markup, { breaks: true });
        emit("updateParsingTime", performance.now() - parsingStart);
        return result;
      } catch (e) {
        if (!(e instanceof Error)) {
          e = new Error(`${e}`);
        }
        emit("error", e as Error);
      }
    };
  },
};
</script>
