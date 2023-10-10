import React from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

import ErrorPage from "./ErrorPage";
import Graph from "./Graph";
import Landing from "./Landing";
import Root from "./Root";

export default function App(): React.ReactElement {
  const router = createBrowserRouter([
    {
      path: "/",
      element: <Root />,
      children: [
        {
          index: true,
          element: <Landing />,
        },
        {
          path: "graph/:startId",
          element: <Graph />,
        },
      ],
      errorElement: (
        <Root>
          <ErrorPage />
        </Root>
      ),
    },
  ]);

  return <RouterProvider router={router} />;
}
