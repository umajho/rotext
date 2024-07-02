import { Component } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { Button } from "../../components/ui/mod";

export default (() => {
  const navigate = useNavigate();

  return (
    <div class="flex h-full justify-center flex-col items-center gap-8">
      <h1 class="text-5xl font-bold">页面不存在</h1>
      <Button type="primary" onClick={() => navigate("/", { replace: true })}>
        返回首页
      </Button>
    </div>
  );
}) satisfies Component;
