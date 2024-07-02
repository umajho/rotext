import { Component } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { Button } from "../../components/ui/mod";

export default (() => {
  const navigate = useNavigate();

  return (
    <div class="flex justify-center flex-col items-center">
      <h1 class="text-5xl font-bold">页面不存在</h1>
      <div class="h-6" />
      <Button type="primary" onClick={() => navigate("/", { replace: true })}>
        返回首页
      </Button>
    </div>
  );
}) satisfies Component;
