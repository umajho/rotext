import { Component } from "solid-js";

export const NavBar: Component = () => {
  return (
    <div class="navbar bg-base-100 shadow-xl rounded-box">
      <div class="flex-1">
        <a class="btn btn-ghost normal-case text-xl">
          <span>
            Rotext<sup>lite</sup> Testground
          </span>
        </a>
      </div>
      <div class="flex-none">
        <ul class="menu menu-horizontal px-1">
          <li>
            <a href="https://github.com/umajho/rotext-lite">代码仓库</a>
          </li>
        </ul>
      </div>
    </div>
  );
};
